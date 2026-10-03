@echo off
setlocal
title Build Gojo/Minato AI mod
cd /d "%~dp0tfm2_custom_ai"

set "GAME=C:\Program Files (x86)\Steam\steamapps\common\Teamfight Manager2"
set "DEST=%GAME%\mods\tfm2_custom_ai"

rem --- find cargo ---------------------------------------------------------
set "CARGO=cargo"
where cargo >nul 2>nul
if errorlevel 1 (
  if exist "%USERPROFILE%\.cargo\bin\cargo.exe" (
    set "CARGO=%USERPROFILE%\.cargo\bin\cargo.exe"
  ) else (
    echo.
    echo Rust is not installed ^(cargo was not found^).
    echo Install it from https://rustup.rs  ^(pick the default options^), then run this file again.
    start "" https://rustup.rs
    pause
    exit /b 1
  )
)

rem --- find rustup ---------------------------------------------------------
set "RUSTUP=rustup"
where rustup >nul 2>nul
if errorlevel 1 if exist "%USERPROFILE%\.cargo\bin\rustup.exe" set "RUSTUP=%USERPROFILE%\.cargo\bin\rustup.exe"

rem --- build --------------------------------------------------------------
rem The GNU toolchain brings its own linker, so Visual Studio is not needed.
set "GNU=stable-x86_64-pc-windows-gnu"
"%RUSTUP%" toolchain list | findstr /b /c:"%GNU%" >nul
if errorlevel 1 (
  echo Installing the Rust GNU toolchain ^(one time, a few hundred MB^) ...
  "%RUSTUP%" toolchain install %GNU% --profile minimal
  if errorlevel 1 goto :buildfail
)
echo Building tfm2_custom_ai ...
"%RUSTUP%" run %GNU% cargo build --release
if errorlevel 1 goto :buildfail

rem --- install into the game ----------------------------------------------
if not exist "%GAME%\TeamfightManager2.exe" (
  echo Game folder not found: "%GAME%"
  echo Copy target\release\tfm2_custom_ai.dll and mod.mod_info into mods\tfm2_custom_ai yourself.
  pause
  exit /b 1
)
if not exist "%DEST%" mkdir "%DEST%"
copy /y "target\release\tfm2_custom_ai.dll" "%DEST%\tfm2_custom_ai.dll" >nul || goto :buildfail
echo.
echo BUILD FAILED. Copy the error text above and send it to Claude.
pause
exit /b 1

:copyfail
copy /y "mod.mod_info" "%DEST%\mod.mod_info" >nul || goto :buildfail
echo.
echo BUILD FAILED. Copy the error text above and send it to Claude.
pause
exit /b 1

:copyfail

echo.
echo Installed to "%DEST%"
echo Next: start the game, open the Mod Manager, enable "Custom champion AI (Gojo, Minato)",
echo accept the code-mod warning, then restart the game.
pause
exit /b 0

:buildfail
echo.
echo BUILD FAILED. Copy the error text above and send it to Claude.
pause
exit /b 1

:copyfail
echo.
echo Could not copy into "%DEST%". Close the game ^(the DLL is locked while it runs^) and try again.
pause
exit /b 1
