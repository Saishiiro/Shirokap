@echo off
setlocal
cd /d "%~dp0"
if not exist ..\shirokap.exe (
  echo shirokap.exe est introuvable : lance d abord build.bat a la racine du projet.
  pause
  exit /b 1
)
set ISCC=
if exist "%ProgramFiles(x86)%\Inno Setup 6\ISCC.exe" set ISCC=%ProgramFiles(x86)%\Inno Setup 6\ISCC.exe
if exist "%ProgramFiles%\Inno Setup 6\ISCC.exe" set ISCC=%ProgramFiles%\Inno Setup 6\ISCC.exe
if exist "%LOCALAPPDATA%\Programs\Inno Setup 6\ISCC.exe" set ISCC=%LOCALAPPDATA%\Programs\Inno Setup 6\ISCC.exe
if "%ISCC%"=="" (
  echo Inno Setup 6 est introuvable. Installe-le depuis https://jrsoftware.org/isinfo.php
  pause
  exit /b 1
)
for /f "tokens=3 delims= " %%v in ('findstr /b /c:"version = " ..\Cargo.toml') do (
  set VER=%%~v
  goto :found
)
:found
"%ISCC%" /DMyAppVersion=%VER% Shirokap.iss
if errorlevel 1 (
  echo Echec de la creation de l installateur.
  pause
  exit /b 1
)
echo OK : l installateur est dans le dossier dist
pause
