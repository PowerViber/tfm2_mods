@echo off
rem Pull the latest mods and install them into Teamfight Manager 2 (the editor runs straight from this folder).
rem   Double-click, or:  "Update game and editor.bat" "D:\SteamLibrary\steamapps\common\Teamfight Manager2"
rem   Flags: /nopull (skip git pull), /nopause (no questions, no pause; native\build.bat uses both)
setlocal
title Update Teamfight Manager 2 mods and editor

rem Run from a temp copy: "git pull" may replace this very file, and cmd reads a batch file as it goes.
if defined TFM2_UPDATE_ROOT goto :run
set "TFM2_UPDATE_ROOT=%~dp0"
copy /y "%~f0" "%TEMP%\tfm2_update.bat" >nul
call "%TEMP%\tfm2_update.bat" %* & exit /b

:run
set "ROOT=%TFM2_UPDATE_ROOT:~0,-1%"
set "FAILED="
set "NOPULL="
set "NOPAUSE="
set "GAMEARG="
:args
if "%~1"=="" goto :argsdone
if /i "%~1"=="/nopull" (set "NOPULL=1") else if /i "%~1"=="/nopause" (set "NOPAUSE=1") else set "GAMEARG=%~1"
shift
goto :args
:argsdone

rem --- 1. pull ---------------------------------------------------------------
if defined NOPULL goto :findgame
where git >nul 2>nul || goto :nogit
git -C "%ROOT%" rev-parse --is-inside-work-tree >nul 2>nul || goto :nogit
echo Pulling the latest version ...
git -C "%ROOT%" pull --ff-only
if not errorlevel 1 goto :findgame
echo.
echo git pull did not work ^(no internet, or files changed here^). See the message above.
choice /m "Install the files already in this folder anyway"
if errorlevel 2 goto :end
goto :findgame
:nogit
echo git was not found ^(or this folder is not a git clone^): installing the files already in this folder.
echo To get new versions, download the repository again from GitHub first.

rem --- 2. find the game ------------------------------------------------------
:findgame
set "GAME=%GAMEARG%"
if not "%GAME%"=="" goto :checkgame
set "GAME=C:\Program Files (x86)\Steam\steamapps\common\Teamfight Manager2"
if exist "%GAME%\bundle.game_data" goto :gamefound
set "GAME=C:\Program Files (x86)\Steam\steamapps\common\Teamfight Manager 2"
:checkgame
if exist "%GAME%\bundle.game_data" goto :gamefound
echo.
echo Teamfight Manager 2 was not found at "%GAME%".
echo In Steam: right-click the game, Manage, Browse local files, and copy that folder's path.
set "GAME="
set /p "GAME=Paste the game folder here (Enter to cancel): "
if not defined GAME goto :end
set "GAME=%GAME:"=%"
goto :checkgame
:gamefound
echo Game: "%GAME%"

rem --- 3. the game must be closed (it locks the native DLL) -------------------
:checkrunning
tasklist /fo csv /nh | findstr /i "teamfight" >nul
if errorlevel 1 goto :closed
echo.
echo Teamfight Manager 2 is running. Close it, then press a key.
if defined NOPAUSE goto :running_fail
pause >nul
goto :checkrunning
:running_fail
set "FAILED=1"
echo Close the game and run this again.
goto :end
:closed

rem --- 4. back up, then install every mod folder ------------------------------
set "STAMP="
for /f %%t in ('powershell -NoProfile -Command "Get-Date -Format yyyyMMdd_HHmmss"') do set "STAMP=%%t"
if not defined STAMP set "STAMP=%RANDOM%"
set "BACKUP=%ROOT%\backups\game_mods_%STAMP%"
if not exist "%GAME%\mods\" mkdir "%GAME%\mods"
echo.
echo Installing into "%GAME%\mods"  (the old files go to backups\game_mods_%STAMP%)
pushd "%ROOT%\mods"
for /d %%m in (*) do call :install "%%m"
popd

rem Levi moved into tfm2_custom: a separate tfm2_levi would load him twice
if not exist "%GAME%\mods\tfm2_levi\" goto :nolevi
xcopy "%GAME%\mods\tfm2_levi" "%BACKUP%\tfm2_levi\" /E /I /Y /Q /R /H >nul
if errorlevel 1 (set "FAILED=1") else rmdir /s /q "%GAME%\mods\tfm2_levi"
if not exist "%GAME%\mods\tfm2_levi\" echo   removed tfm2_levi ^(Levi is in tfm2_custom now; kept in the backup^)
:nolevi
rem round 91: older standalone copies of the custom champions (tfm2_isliid, tfm2_gundam, ...) would override the new data
pushd "%GAME%\mods"
for /d %%m in (*) do call :stale "%%m"
popd
rem files a newer version replaced (round 89: the engraving strokes moved to engrave_t0..t3)
del /q "%GAME%\mods\tfm2_custom\vfx\engraving_colors#sheet.png" "%GAME%\mods\tfm2_custom\vfx\engraving_colors#anim.fanim" 2>nul

rem --- 5. check: every file of the custom champions and the native DLL, byte for byte ---------------
echo.
fc /b "%ROOT%\mods\tfm2_custom_ai\tfm2_custom_ai.dll" "%GAME%\mods\tfm2_custom_ai\tfm2_custom_ai.dll" >nul
if errorlevel 1 (set "FAILED=1" & echo The native DLL did not copy. Is the game still open?) else echo Native DLL installed and checked.
set "CHECKED=0"
set "BAD=0"
call :checkfile . mod.mod_info
for %%d in (champion champions vfx text) do call :checkdir %%d
if "%BAD%"=="0" (echo All %CHECKED% files of the custom champions match.) else (set "FAILED=1" & echo %BAD% of %CHECKED% custom champion files do NOT match.)
call :version tfm2_custom_ai
call :version tfm2_custom
echo.
if defined FAILED goto :failed
echo Done. Start the game, check under Mods that the mods are enabled (tfm2_custom_ai needs one restart).
echo If the editor is open, close it and start it again: it runs straight from this folder.
echo In game, mods\tfm2_custom_ai\gundam_log.txt and isliid_log.txt name the native version that ran.
if defined NOPAUSE goto :end
choice /m "Start the editor now"
if errorlevel 2 goto :end
start "TFM2 Database Editor" "%ROOT%\editor\Start Editor.bat"
goto :end

:failed
echo Some files could not be copied (see above). Close the game and the editor and run this again.
echo Nothing was lost: the previous files are in "%BACKUP%".
goto :end

rem --- compare one folder of tfm2_custom with the installed copy ----------------
:checkdir
if not exist "%ROOT%\mods\tfm2_custom\%~1\" exit /b
pushd "%ROOT%\mods\tfm2_custom\%~1"
for %%f in (*) do call :checkfile "%~1" "%%f"
popd
exit /b
:checkfile
set /a CHECKED+=1
fc /b "%ROOT%\mods\tfm2_custom\%~1\%~2" "%GAME%\mods\tfm2_custom\%~1\%~2" >nul 2>nul
if errorlevel 1 (set /a BAD+=1 & echo   MISMATCH tfm2_custom\%~1\%~2)
exit /b

rem --- an installed folder holding an old copy of a custom champion goes to the backup ----------------
:stale
if /i "%~1"=="tfm2_custom" exit /b
set "OLD="
for %%c in (tfm2_isliid_emperor tfm2_gundam_aegis_zero tfm2_levi_levi) do if exist "%GAME%\mods\%~1\champion\%%c.data_champion" set "OLD=%%c"
if not defined OLD exit /b
xcopy "%GAME%\mods\%~1" "%BACKUP%\%~1\" /E /I /Y /Q /R /H >nul
if errorlevel 1 (set "FAILED=1" & echo   FAILED to move the old %~1) else rmdir /s /q "%GAME%\mods\%~1"
if not exist "%GAME%\mods\%~1\" echo   removed %~1 ^(an old copy of %OLD%; kept in the backup^)
exit /b

rem --- print a mod's version (the first "version" in its mod.mod_info) -------
:version
set "V="
for /f "tokens=1,2 delims=:, " %%a in ('findstr /c:"\"version\"" "%GAME%\mods\%~1\mod.mod_info"') do if not defined V set "V=%%~b"
echo   %~1 %V%
exit /b

rem --- install one mod folder (only real mods: they have a mod.mod_info) ------
:install
set "NAME=%~1"
if not exist "%ROOT%\mods\%NAME%\mod.mod_info" exit /b
if not exist "%GAME%\mods\%NAME%\" goto :copymod
xcopy "%GAME%\mods\%NAME%" "%BACKUP%\%NAME%\" /E /I /Y /Q /R /H >nul
if errorlevel 1 goto :installfail
:copymod
if /i "%NAME%"=="tfm2_custom_ai" goto :install_ai
xcopy "%ROOT%\mods\%NAME%" "%GAME%\mods\%NAME%\" /E /I /Y /Q /R >nul
if errorlevel 1 goto :installfail
echo   updated %NAME%
exit /b
:install_ai
rem the DLL and its mod info always; the Map tab's plans (tactics) and the map dump only when the game has none yet
if not exist "%GAME%\mods\%NAME%\" mkdir "%GAME%\mods\%NAME%"
for %%f in (tfm2_custom_ai.dll mod.mod_info) do copy /y "%ROOT%\mods\%NAME%\%%f" "%GAME%\mods\%NAME%\" >nul
fc /b "%ROOT%\mods\%NAME%\mod.mod_info" "%GAME%\mods\%NAME%\mod.mod_info" >nul
if errorlevel 1 goto :installfail
set "PLANS=new"
if exist "%GAME%\mods\%NAME%\tactics.txt" set "PLANS=kept"
if exist "%GAME%\mods\%NAME%\tactics.json" set "PLANS=kept"
pushd "%ROOT%\mods\%NAME%"
for %%f in (*) do call :addnew "%%f"
popd
if "%PLANS%"=="kept" (echo   updated %NAME% ^(your Map tab plans were kept^)) else echo   updated %NAME%
exit /b
:addnew
rem a file the game folder doesn't have yet; the Map tab's tactics.json and tactics.txt only ever come as a pair
if exist "%GAME%\mods\%NAME%\%~1" exit /b
if /i "%~1"=="tactics.json" if "%PLANS%"=="kept" exit /b
if /i "%~1"=="tactics.txt" if "%PLANS%"=="kept" exit /b
copy "%~1" "%GAME%\mods\%NAME%\" >nul
exit /b
:installfail
set "FAILED=1"
echo   FAILED  %NAME%
exit /b

:end
echo.
if not defined NOPAUSE pause
if defined FAILED exit /b 1
exit /b 0
