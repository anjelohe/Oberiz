Option Explicit

Dim shell, fso, appDir, oberizHome
Set shell = CreateObject("WScript.Shell")
Set fso = CreateObject("Scripting.FileSystemObject")
appDir = fso.GetParentFolderName(WScript.ScriptFullName)
oberizHome = shell.ExpandEnvironmentStrings("%LOCALAPPDATA%") & "\\Oberiz"

EnsureFolder oberizHome
EnsureFolder oberizHome & "\\data"
EnsureFolder oberizHome & "\\config"
EnsureFolder oberizHome & "\\config\\indexers"
EnsureFolder oberizHome & "\\config\\indexers\\custom"
EnsureFolder oberizHome & "\\config\\indexers\\upstream"

shell.Environment("PROCESS")("OBERIZ_DATA_DIR") = oberizHome & "\\data"
shell.Environment("PROCESS")("OBERIZ_CONFIG_DIR") = oberizHome & "\\config"
shell.Environment("PROCESS")("OBERIZ_STATIC_DIR") = appDir & "\\frontend"
shell.CurrentDirectory = appDir
shell.Run Chr(34) & appDir & "\\Oberiz.exe" & Chr(34), 0, False
shell.Run "http://127.0.0.1:2032", 1, False

Sub EnsureFolder(path)
  If Not fso.FolderExists(path) Then fso.CreateFolder(path)
End Sub
