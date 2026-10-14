//! Runs Oberiz as a real Windows Service (Service Control Manager managed):
//! starts at boot, survives logoff and doesn't need a signed-in user, and is
//! controlled through `services.msc` / `sc.exe` like any other service. This
//! is what `--install-service` registers; the portable ZIP / double-click
//! flow (plain `Oberiz.exe`, no arguments) never touches this module and
//! keeps behaving exactly as it always has.
use std::{
    ffi::OsString,
    io::Write,
    path::Path,
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::sync::Notify;
use windows_service::{
    define_windows_service,
    service::{
        ServiceAccess, ServiceControl, ServiceControlAccept, ServiceErrorControl, ServiceExitCode,
        ServiceInfo, ServiceStartType, ServiceState, ServiceStatus, ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
    service_manager::{ServiceManager, ServiceManagerAccess},
};

pub const SERVICE_NAME: &str = "Oberiz";
const SERVICE_DISPLAY_NAME: &str = "Oberiz";
const SERVICE_DESCRIPTION: &str = "Oberiz media automation server";

define_windows_service!(ffi_service_main, service_main);

/// Entry point when launched by the Service Control Manager as
/// `Oberiz.exe --service`. Blocks for the lifetime of the service.
pub fn run_as_service() -> anyhow::Result<()> {
    service_dispatcher::start(SERVICE_NAME, ffi_service_main)?;
    Ok(())
}

fn service_main(_arguments: Vec<OsString>) {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(run_service)) {
        Ok(Err(error)) => {
            tracing::error!(error = %error, "windows service exited with an error");
            write_service_error(&error);
        }
        Err(payload) => {
            let message = payload
                .downcast_ref::<&str>()
                .map(ToString::to_string)
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "non-string panic payload".to_string());
            write_service_message(&format!("service panicked: {message}"));
        }
        Ok(Ok(())) => write_service_message("service stopped normally"),
    }
}

/// Services have no visible console, so keep their startup failures in a
/// machine-wide log that an administrator can inspect after SCM reports a
/// generic exit code. Logging must never prevent the service from reporting
/// its real error to the Service Control Manager.
fn write_service_message(message: &str) {
    let program_data =
        std::env::var("ProgramData").unwrap_or_else(|_| r"C:\ProgramData".to_string());
    let directory = Path::new(&program_data).join("Oberiz").join("data");
    let Ok(()) = std::fs::create_dir_all(&directory) else {
        return;
    };
    let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(directory.join("service.log"))
    else {
        return;
    };
    let _ = writeln!(file, "{:?}: {message}", SystemTime::now());
}

fn write_service_error(error: &anyhow::Error) {
    write_service_message(&format!("service failed: {error:#}"));
}

fn run_service() -> anyhow::Result<()> {
    let shutdown = Arc::new(Notify::new());
    let shutdown_for_handler = shutdown.clone();

    let status_handle = service_control_handler::register(SERVICE_NAME, move |control| {
        match control {
            // The SCM's own Stop/Shutdown request — the normal way this
            // service is expected to stop.
            ServiceControl::Stop | ServiceControl::Shutdown => {
                shutdown_for_handler.notify_one();
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    })?;

    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;
    let runtime = tokio::runtime::Runtime::new()?;
    let result = runtime.block_on(async move {
        let state = crate::build_app_state().await?;
        crate::spawn_schedulers(&state);
        crate::run_server(state, async move { shutdown.notified().await }).await
    });

    let _ = status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(if result.is_ok() { 0 } else { 1 }),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    });

    result
}

/// Registers the service pointing back at this same executable with
/// `--service`, and grants the interactively logged-in user (the tray
/// helper, running unprivileged) the right to start/stop it without a UAC
/// prompt every time. Intended to be run once by the (elevated) installer.
pub fn install() -> anyhow::Result<()> {
    let manager = ServiceManager::local_computer(
        None::<&str>,
        ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE,
    )?;
    let exe_path = std::env::current_exe()?;
    let service_info = ServiceInfo {
        name: OsString::from(SERVICE_NAME),
        display_name: OsString::from(SERVICE_DISPLAY_NAME),
        service_type: ServiceType::OWN_PROCESS,
        start_type: ServiceStartType::AutoStart,
        error_control: ServiceErrorControl::Normal,
        executable_path: exe_path,
        launch_arguments: vec![OsString::from("--service")],
        dependencies: vec![],
        account_name: None, // runs as LocalSystem
        account_password: None,
    };
    let access = ServiceAccess::CHANGE_CONFIG | ServiceAccess::START;
    // An installer upgrade keeps the existing service registration. Reuse it
    // and point it at the freshly installed executable instead of failing
    // with ERROR_SERVICE_EXISTS and silently leaving the old binary active.
    let service = match manager.open_service(SERVICE_NAME, access) {
        Ok(service) => {
            service.change_config(&service_info)?;
            service
        }
        Err(windows_service::Error::Winapi(error)) if error.raw_os_error() == Some(1060) => {
            manager.create_service(&service_info, access)?
        }
        Err(error) => return Err(error.into()),
    };
    service.set_description(SERVICE_DESCRIPTION)?;
    // The Registry key only exists after `create_service`; configure the
    // process environment before starting it so its very first boot uses the
    // persistent ProgramData locations rather than System32-relative paths.
    configure_service_environment()?;
    if let Err(windows_service::Error::Winapi(error)) = service.start::<OsString>(&[]) {
        // A repair install may run while Oberiz is already serving requests.
        // The updated configuration is still valid; no second start is needed.
        if error.raw_os_error() != Some(1056) {
            return Err(error.into());
        }
    }
    drop(service);

    grant_authenticated_users_start_stop()
}

/// A LocalSystem service has no meaningful per-user `%LOCALAPPDATA%` (unlike
/// the hidden-process portable flow), so its data/config live under the
/// machine-wide `%ProgramData%\Oberiz` instead — the same idea as the Linux
/// package's `/var/lib/oberiz`. Set as the service's own environment via the
/// registry, since `windows-service` has no API for that.
fn configure_service_environment() -> anyhow::Result<()> {
    let exe_dir = std::env::current_exe()?
        .parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| anyhow::anyhow!("could not determine the executable's directory"))?;
    let program_data =
        std::env::var("ProgramData").unwrap_or_else(|_| r"C:\ProgramData".to_string());
    let data_dir = format!("{program_data}\\Oberiz\\data");
    let config_dir = format!("{program_data}\\Oberiz\\config");
    let static_dir = exe_dir.join("frontend").display().to_string();

    std::fs::create_dir_all(&data_dir)?;
    std::fs::create_dir_all(format!("{config_dir}\\indexers\\custom"))?;
    std::fs::create_dir_all(format!("{config_dir}\\indexers\\upstream"))?;
    migrate_legacy_user_data(Path::new(&data_dir), Path::new(&config_dir))?;

    let environment = format!(
        "OBERIZ_DATA_DIR={data_dir}\\0OBERIZ_CONFIG_DIR={config_dir}\\0OBERIZ_STATIC_DIR={static_dir}\\0"
    );
    let output = std::process::Command::new("reg")
        .args([
            "add",
            &format!(r"HKLM\SYSTEM\CurrentControlSet\Services\{SERVICE_NAME}"),
            "/v",
            "Environment",
            "/t",
            "REG_MULTI_SZ",
            "/d",
            &environment,
            "/f",
        ])
        .output()?;
    if !output.status.success() {
        anyhow::bail!(
            "reg add failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

/// v1.0.0 stored Windows-installed data below the current user's
/// `%LOCALAPPDATA%`. Move that state to the machine-wide directory used by
/// the service without replacing any files that the new installer already
/// placed there. This makes upgrading opt-in safe: a fresh service starts
/// cleanly, while an existing user's library, settings and custom indexers
/// continue to work.
fn migrate_legacy_user_data(data_dir: &Path, config_dir: &Path) -> anyhow::Result<()> {
    let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") else {
        return Ok(());
    };
    let legacy_root = Path::new(&local_app_data).join("Oberiz");
    let legacy_data = legacy_root.join("data");
    let legacy_config = legacy_root.join("config");

    if !data_dir.join("oberiz.db").exists() {
        copy_directory_missing(&legacy_data, data_dir)?;
    }
    copy_directory_missing(&legacy_config, config_dir)
}

fn copy_directory_missing(source: &Path, destination: &Path) -> anyhow::Result<()> {
    if !source.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if source_path.is_dir() {
            std::fs::create_dir_all(&destination_path)?;
            copy_directory_missing(&source_path, &destination_path)?;
        } else if !destination_path.exists() {
            std::fs::copy(&source_path, &destination_path)?;
        }
    }
    Ok(())
}

/// Stops (if running) and removes the service. Does not touch the tray
/// helper's Startup-folder shortcut — that is the installer/uninstaller's
/// responsibility, since it knows the actual per-user shortcut path.
pub fn uninstall() -> anyhow::Result<()> {
    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)?;
    let service = manager.open_service(
        SERVICE_NAME,
        ServiceAccess::STOP | ServiceAccess::DELETE | ServiceAccess::QUERY_STATUS,
    )?;
    if let Ok(status) = service.query_status()
        && status.current_state != ServiceState::Stopped
    {
        let _ = service.stop();
        for _ in 0..40 {
            std::thread::sleep(Duration::from_millis(250));
            if matches!(service.query_status(), Ok(s) if s.current_state == ServiceState::Stopped) {
                break;
            }
        }
    }
    service.delete()?;
    Ok(())
}

/// `windows-service` doesn't expose the security descriptor directly, so this
/// shells out to the standard `sc.exe sdset`. The SDDL keeps full control for
/// LocalSystem (SY) and Administrators (BA) — the same as the Windows
/// default — and additionally grants Authenticated Users (AU) the rights to
/// start, stop, and query the service, and Interactive Users (IU) read
/// access, so the unprivileged tray helper can control it.
fn grant_authenticated_users_start_stop() -> anyhow::Result<()> {
    const SDDL: &str = "D:(A;;CCLCSWRPWPDTLOCRRC;;;SY)(A;;CCDCLCSWRPWPDTLOCRRC;;;BA)(A;;CCLCSWRPWP;;;AU)(A;;CCLCSWLOCRRC;;;IU)";
    let output = std::process::Command::new("sc")
        .args(["sdset", SERVICE_NAME, SDDL])
        .output()?;
    if !output.status.success() {
        anyhow::bail!(
            "sc sdset failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}
