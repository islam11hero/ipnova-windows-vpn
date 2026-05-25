@echo off
setlocal
cd /d "%~dp0"
if exist "%~dp0Install IPNOVA.bat" (
    call "%~dp0Install IPNOVA.bat"
    exit /b %ERRORLEVEL%
)
powershell -NoProfile -ExecutionPolicy Bypass -STA -File "%~dp0Setup-IPNOVA.ps1"
exit /b %ERRORLEVEL%
