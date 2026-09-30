Option Explicit

' Ensure the service is running before opening the web interface. This also
' makes the shortcut recover Oberiz after the tray menu's "Close Oberiz".
Dim shell, request, attempt
Set shell = CreateObject("WScript.Shell")
shell.Run "sc.exe start Oberiz", 0, True

For attempt = 1 To 40
  On Error Resume Next
  Set request = CreateObject("WinHttp.WinHttpRequest.5.1")
  request.SetTimeouts 250, 250, 250, 250
  request.Open "GET", "http://127.0.0.1:2032/api/health", False
  request.Send
  If Err.Number = 0 And request.Status = 200 Then Exit For
  Err.Clear
  On Error GoTo 0
  WScript.Sleep 250
Next

shell.Run "http://127.0.0.1:2032", 1, False
