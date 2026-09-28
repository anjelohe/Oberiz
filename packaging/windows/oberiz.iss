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
Source: "{#SourceRoot}\config\*"; DestDir: "{localappdata}\Oberiz\config"; Flags: onlyifdoesntexist recursesubdirs createallsubdirs
Source: "{#SourceRoot}\Start-Oberiz-Installed.bat"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceRoot}\Start-Oberiz-Installed.vbs"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceRoot}\oberiz.ico"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\Oberiz"; Filename: "{sys}\wscript.exe"; Parameters: """{app}\Start-Oberiz-Installed.vbs"""; WorkingDir: "{app}"; IconFilename: "{app}\oberiz.ico"
Name: "{autodesktop}\Oberiz"; Filename: "{sys}\wscript.exe"; Parameters: """{app}\Start-Oberiz-Installed.vbs"""; WorkingDir: "{app}"; IconFilename: "{app}\oberiz.ico"; Tasks: desktopicon

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional shortcuts:"

[Run]
Filename: "{sys}\wscript.exe"; Parameters: """{app}\Start-Oberiz-Installed.vbs"""; Description: "Start Oberiz"; Flags: nowait postinstall skipifsilent
