@echo off
setlocal
cd /d "%~dp0"
rem Sans ca, les chemins de ce PC (dossier utilisateur, donc le nom de session Windows) restent
rem ecrits dans l'exe, dans les messages d'erreur internes des bibliotheques : on les remplace.
set "REMAP='--remap-path-prefix=%USERPROFILE%=~'"
if defined CARGO_HOME set "REMAP=%REMAP%,'--remap-path-prefix=%CARGO_HOME%=cargo'"
set "REMAP=%REMAP%,'--remap-path-prefix=%CD%=shirokap'"
cargo build --release --config "target.'cfg(windows)'.rustflags=[%REMAP%]"
if errorlevel 1 (
  echo Echec de la compilation.
  pause
  exit /b 1
)
copy /Y target\release\shirokap.exe shirokap.exe >nul && echo OK : shirokap.exe est pret
pause
