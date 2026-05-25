@echo off
setlocal EnableExtensions
title IPNOVA VPN Install
cd /d "%~dp0"

echo.
echo ========================================
echo        IPNOVA VPN - Installation
echo ========================================
echo.
echo Folder: %CD%
echo.

if exist "%~dp0Install IPNOVA (Setup).exe" (
    echo Running official installer...
    "%~dp0Install IPNOVA (Setup).exe"
    goto :finish
)

if exist "%~dp0IPNOVA VPN_0.1.0_x64-setup.exe" (
    echo Running official installer...
    "%~dp0IPNOVA VPN_0.1.0_x64-setup.exe"
    goto :finish
)

if exist "%~dp0setup.exe" (
    echo Running setup.exe...
    "%~dp0setup.exe"
    goto :finish
)

echo Running setup script...
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0Setup-IPNOVA.ps1"
if errorlevel 1 goto :failed
goto :finish

:failed
echo.
echo INSTALLATION FAILED.
echo - Open README.txt in this folder
echo - Or ask for a ZIP built with build-windows-installer.ps1 on Windows
echo.
pause
exit /b 1

:finish
echo.
echo Done.
timeout /t 5 >nul
exit /b 0
