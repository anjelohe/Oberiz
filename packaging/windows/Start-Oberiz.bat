@echo off
setlocal
cd /d "%~dp0"

if not exist data mkdir data
if not exist config\indexers\custom mkdir config\indexers\custom
if not exist config\indexers\upstream mkdir config\indexers\upstream

set "OBERIZ_DATA_DIR=%CD%\data"
set "OBERIZ_CONFIG_DIR=%CD%\config"
set "OBERIZ_STATIC_DIR=%CD%\frontend"

start "Oberiz" http://127.0.0.1:2032
Oberiz.exe
