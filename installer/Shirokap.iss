; Script Inno Setup de Shirokap. Lance d'abord build.bat (il crée shirokap.exe à la racine du projet).
#define MyAppName "Shirokap"
#ifndef MyAppVersion
  #define MyAppVersion "0.0.0"
#endif

[Setup]
AppId={{A6B5F0E2-6C57-4C61-9B58-5E0D1B2C7A11}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher=Saishiiro
AppPublisherURL=https://github.com/Saishiiro/Shirokap
VersionInfoVersion={#MyAppVersion}
VersionInfoDescription=Shirokap setup
VersionInfoCompany=Saishiiro
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
OutputDir=..\dist
OutputBaseFilename=Shirokap-Setup-{#MyAppVersion}
Compression=lzma2
SolidCompression=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
UninstallDisplayIcon={app}\shirokap.exe
SetupIconFile=..\assets\logo.ico
WizardStyle=modern
ShowLanguageDialog=auto
CloseApplications=yes

[Languages]
Name: "french"; MessagesFile: "compiler:Languages\French.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[CustomMessages]
french.DesktopIcon=Créer un raccourci sur le Bureau
english.DesktopIcon=Create a desktop shortcut
french.LaunchApp=Lancer Shirokap
english.LaunchApp=Launch Shirokap

[Tasks]
Name: "desktopicon"; Description: "{cm:DesktopIcon}"; Flags: unchecked

[Files]
Source: "..\shirokap.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\LICENSE"; DestDir: "{app}"; DestName: "LICENSE.txt"; Flags: ignoreversion
Source: "..\assets\fonts\LICENSE-Inter.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\assets\fonts\LICENSE-Lucide.txt"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\Shirokap"; Filename: "{app}\shirokap.exe"
Name: "{autodesktop}\Shirokap"; Filename: "{app}\shirokap.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\shirokap.exe"; Description: "{cm:LaunchApp}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
; arrête l'appli et retire le lancement automatique avec Windows ; les captures et réglages sont conservés
Filename: "taskkill"; Parameters: "/F /IM shirokap.exe"; Flags: runhidden; RunOnceId: "StopShirokap"
Filename: "reg"; Parameters: "delete HKCU\Software\Microsoft\Windows\CurrentVersion\Run /v Shirokap /f"; Flags: runhidden; RunOnceId: "RemoveAutostart"
