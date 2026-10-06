@echo off
setlocal
title Build tfm2_custom_ai native mod
cd /d "%~dp0tfm2_custom_ai"


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
rem Round 91: the new DLL goes into this repo's mods\tfm2_custom_ai, then the updater installs EVERYTHING into the game:
rem the DLL, the champions' data and sprite sheets (mods\tfm2_custom: Isliid, Aegis Zero, Levi, Minato) and the other
rem mods, with a backup, the removal of old duplicate copies and a byte-for-byte check. (This used to copy only the DLL,
rem so the game ran new native code against old data: missing effects and sprites.)
copy /y "target\release\tfm2_custom_ai.dll" "%~dp0..\mods\tfm2_custom_ai\tfm2_custom_ai.dll" >nul || goto :copyfail
copy /y "mod.mod_info" "%~dp0..\mods\tfm2_custom_ai\mod.mod_info" >nul || goto :copyfail
echo Built. Installing everything into the game ...
call "%~dp0..\Update game and editor.bat" /nopull /nopause
if errorlevel 1 goto :installfail

echo.
echo Next: start the game, open the Mod Manager, enable "Gojo & Minato rules (native)" and the champion mods,
echo accept the code-mod warning, then restart the game.
pause
exit /b 0

:installfail
echo.
echo The install did not finish (see above). Close the game and run this again.
pause
exit /b 1

:buildfail
echo.
echo BUILD FAILED. Copy the error text above and send it to Claude.
pause
exit /b 1

:copyfail
echo.
echo Could not copy the new DLL into this repo's mods\tfm2_custom_ai. Close the game and the editor and try again.
pause
exit /b 1
