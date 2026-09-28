@echo off
setlocal
cd /d "%~dp0"

set "OBERIZ_HOME=%LOCALAPPDATA%\Oberiz"
if not exist "%OBERIZ_HOME%\data" mkdir "%OBERIZ_HOME%\data"
if not exist "%OBERIZ_HOME%\config\indexers\custom" mkdir "%OBERIZ_HOME%\config\indexers\custom"
if not exist "%OBERIZ_HOME%\config\indexers\upstream" mkdir "%OBERIZ_HOME%\config\indexers\upstream"

set "OBERIZ_DATA_DIR=%OBERIZ_HOME%\data"
set "OBERIZ_CONFIG_DIR=%OBERIZ_HOME%\config"
set "OBERIZ_STATIC_DIR=%CD%\frontend"

start "Oberiz" http://127.0.0.1:2032
Oberiz.exe
