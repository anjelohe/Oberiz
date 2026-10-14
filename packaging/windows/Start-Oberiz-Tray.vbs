Option Explicit

' Started at user logon (Startup folder shortcut created by the installer).
' Shows the notification-area icon that opens Oberiz and starts/stops the
' Windows Service; it does not run the web server itself.
Dim shell, appDir
Set shell = CreateObject("WScript.Shell")
appDir = CreateObject("Scripting.FileSystemObject").GetParentFolderName(WScript.ScriptFullName)
shell.Run Chr(34) & appDir & "\Oberiz.exe" & Chr(34) & " --tray", 0, False
