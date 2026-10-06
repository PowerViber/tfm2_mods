@echo off
rem Builds the native DLL and installs everything into the game: TFM2 Mod Manager.exe (in the repo folder) does it all
rem (build with Rust, put the DLL in mods\tfm2_custom_ai, install every mod with a backup, check the install byte for
rem byte), and explains any error with WHAT / WHY / HOW. Its log is in the repo's logs folder.
"%~dp0..\TFM2 Mod Manager.exe" --update
