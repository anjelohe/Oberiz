Option Explicit

Dim shell, appDir, dataDir
Set shell = CreateObject("WScript.Shell")
appDir = CreateObject("Scripting.FileSystemObject").GetParentFolderName(WScript.ScriptFullName)
dataDir = appDir & "\\data"

EnsureFolder dataDir
EnsureFolder appDir & "\\config\\indexers\\custom"
EnsureFolder appDir & "\\config\\indexers\\upstream"

shell.Environment("PROCESS")("OBERIZ_DATA_DIR") = dataDir
shell.Environment("PROCESS")("OBERIZ_CONFIG_DIR") = appDir & "\\config"
shell.Environment("PROCESS")("OBERIZ_STATIC_DIR") = appDir & "\\frontend"
shell.CurrentDirectory = appDir
shell.Run Chr(34) & appDir & "\\Oberiz.exe" & Chr(34), 0, False
shell.Run "http://127.0.0.1:2032", 1, False

Sub EnsureFolder(path)
  Dim fso
  Set fso = CreateObject("Scripting.FileSystemObject")
  If Not fso.FolderExists(path) Then fso.CreateFolder(path)
End Sub
