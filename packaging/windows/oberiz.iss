#ifndef SourceRoot
  #error SourceRoot must point to the prepared Windows package directory.
#endif
#ifndef MyAppVersion
  #error MyAppVersion must be supplied when compiling the installer.
#endif

#define MyAppName "Oberiz"
#define MyAppPublisher "Oberiz"
#define MyAppExeName "Oberiz.exe"

[Setup]
AppId={{2F4D244D-F29E-4CE7-9CE3-F3F635B2DCED}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
DefaultDirName={autopf}\Oberiz
DefaultGroupName=Oberiz
DisableProgramGroupPage=yes
OutputDir={#SourceRoot}\..
OutputBaseFilename=oberiz-{#MyAppVersion}-windows-x86_64-setup
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=admin
UninstallDisplayIcon={app}\{#MyAppExeName}
SetupIconFile={#SourceRoot}\oberiz.ico

[Files]
Source: "{#SourceRoot}\Oberiz.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceRoot}\frontend\*"; DestDir: "{app}\frontend"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "{#SourceRoot}\config\*"; DestDir: "{commonappdata}\Oberiz\config"; Flags: onlyifdoesntexist recursesubdirs createallsubdirs
Source: "{#SourceRoot}\Start-Oberiz-Installed.bat"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceRoot}\Start-Oberiz-Installed.vbs"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceRoot}\Start-Oberiz-Tray.vbs"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceRoot}\oberiz.ico"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\Oberiz"; Filename: "{sys}\wscript.exe"; Parameters: """{app}\Start-Oberiz-Installed.vbs"""; WorkingDir: "{app}"; IconFilename: "{app}\oberiz.ico"
Name: "{autodesktop}\Oberiz"; Filename: "{sys}\wscript.exe"; Parameters: """{app}\Start-Oberiz-Installed.vbs"""; WorkingDir: "{app}"; IconFilename: "{app}\oberiz.ico"; Tasks: desktopicon
Name: "{userstartup}\Oberiz Tray"; Filename: "{sys}\wscript.exe"; Parameters: """{app}\Start-Oberiz-Tray.vbs"""; WorkingDir: "{app}"; IconFilename: "{app}\oberiz.ico"

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional shortcuts:"

[Code]
procedure CurStepChanged(CurStep: TSetupStep);
var
  ResultCode: Integer;
begin
  if CurStep = ssInstall then begin
    { An existing service keeps Oberiz.exe locked. Stop it before replacing
      application files; the [Run] entry below reconfigures and starts it. }
    Exec(ExpandConstant('{sys}\sc.exe'), 'stop Oberiz', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
    Sleep(1000);
  end;
end;

[Run]
; Registers and starts the Oberiz Windows Service (runs at boot, no login
; required) and grants the logged-in user's tray icon start/stop rights.
Filename: "{app}\Oberiz.exe"; Parameters: "--install-service"; StatusMsg: "Installing the Oberiz service..."; Flags: runhidden
; Permit Oberiz from other devices on trusted local networks without a first-use
; firewall prompt. Public networks remain blocked.
Filename: "{sys}\netsh.exe"; Parameters: "advfirewall firewall add rule name=""Oberiz local network"" dir=in action=allow program=""{app}\Oberiz.exe"" enable=yes profile=private protocol=TCP localport=2032"; Flags: runhidden
; The tray helper for this session, without waiting for the next logon.
Filename: "{sys}\wscript.exe"; Parameters: """{app}\Start-Oberiz-Tray.vbs"""; WorkingDir: "{app}"; Flags: nowait runhidden
Filename: "{sys}\wscript.exe"; Parameters: """{app}\Start-Oberiz-Installed.vbs"""; Description: "Open Oberiz"; Flags: nowait postinstall skipifsilent

[UninstallRun]
Filename: "{sys}\netsh.exe"; Parameters: "advfirewall firewall delete rule name=""Oberiz local network"" program=""{app}\Oberiz.exe"" protocol=TCP localport=2032"; Flags: runhidden
Filename: "{app}\Oberiz.exe"; Parameters: "--uninstall-service"; Flags: runhidden
