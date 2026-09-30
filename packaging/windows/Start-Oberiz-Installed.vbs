Option Explicit

' The Oberiz Windows Service is already running in the background
' (installed and started by "Oberiz.exe --install-service" during setup);
' this shortcut only needs to open the web interface.
CreateObject("WScript.Shell").Run "http://127.0.0.1:2032", 1, False
