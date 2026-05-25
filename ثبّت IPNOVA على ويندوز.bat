@echo off
title IPNOVA - Build installer on Windows
cd /d "%~dp0"
echo.
echo This will BUILD and create IPNOVA-VPN-Setup.exe on your Desktop.
echo Requires: Node.js 20 + Rust (see docs/INSTALL-WINDOWS-AR.md)
echo.
pause
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\INSTALL-ON-WINDOWS.ps1"
pause
