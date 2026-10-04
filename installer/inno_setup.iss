; Inno Setup Script for typingforge (VS Code-Grade "Next, Next, Next, Finish" Windows Installer)
; Compiles a lightweight, professional graphical installer that requires zero admin rights.

#define MyAppName "typingforge"
#define MyAppVersion "0.1.18"
#define MyAppPublisher "Saboor Hamedi"
#define MyAppURL "https://github.com/Saboor-Hamedi/typingforge"
#define MyAppExeName "forgetyping.exe"

[Setup]
; Basic Application Metadata
AppId={{E76D02A9-3C5E-4B02-86DC-69BE8F4B892D}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}

; Installation Paths (VS Code User-Level Default: Installs seamlessly into user profile without UAC prompt)
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
UsedUserAreasWarning=no

; Automatic Process Handling (Closes running instance during update/reinstall)
CloseApplications=yes
CloseApplicationsFilter=*.exe
RestartApplications=no

; Output Configuration
OutputDir=..\target\installer
OutputBaseFilename=forgetyping-windows-setup
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern

; Visual & Architecture Settings
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
ArchitecturesInstallIn64BitMode=x64
UninstallDisplayIcon={app}\{#MyAppExeName}
UninstallDisplayName={#MyAppName}

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
; Release Executable & Docs
Source: "..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
; Auto-Launch after completion
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
