@echo off
title TFM2 Database Editor
rem Tip: "TFM2 Mod Manager.exe" in the repo folder starts this editor too, and also updates and installs the mods.
cd /d "%~dp0"
where node >nul 2>nul
if errorlevel 1 (
  echo Node.js was not found, so the editor will open in standalone mode.
  echo In standalone mode you pick files yourself and saves are downloaded.
  echo Install Node.js from https://nodejs.org for automatic save detection and backups.
  start "" "%~dp0index.html"
  pause
  exit /b
)
node "%~dp0server.js" %*
pause
