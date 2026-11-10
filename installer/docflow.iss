; Instalador de DocFlow para Windows (Inno Setup 6).
; Compilar:  ISCC /DAppVersion=0.1.0 installer\docflow.iss   (lo hace build.ps1 / build.sh)
; Requiere haber compilado antes:  npm run tauri build -- --no-bundle

#define AppName "DocFlow"
#ifndef AppVersion
  #define AppVersion "0.1.0"
#endif
#define AppExe "docflow.exe"

[Setup]
AppId={{7B3F2C1E-5D4A-4E8B-9C6A-DF10AC0FF10E}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher=DocFlow
AppComments=Automatización documental local y segura
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
SetupIconFile=..\src-tauri\icons\icon.ico
UninstallDisplayIcon={app}\{#AppExe}
OutputDir=..\dist-release
OutputBaseFilename=DocFlow-Setup-{#AppVersion}
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
; Instala solo para el usuario sin pedir administrador; permite elegir "para todos".
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog

[Languages]
Name: "spanish"; MessagesFile: "compiler:Languages\Spanish.isl"

[Tasks]
Name: "desktopicon"; Description: "Crear un acceso directo en el &escritorio"; GroupDescription: "Accesos directos:"

[Files]
Source: "..\src-tauri\target\release\{#AppExe}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\src-tauri\icons\icon.ico"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExe}"; IconFilename: "{app}\icon.ico"; Tasks: desktopicon
Name: "{autoprograms}\{#AppName}"; Filename: "{app}\{#AppExe}"; IconFilename: "{app}\icon.ico"

[Run]
Filename: "{app}\{#AppExe}"; Description: "Abrir {#AppName}"; Flags: nowait postinstall skipifsilent

[Code]
// DocFlow usa WebView2 (viene con Windows 11 y Windows 10 actualizado).
function WebView2Installed: Boolean;
var v: String;
begin
  Result := RegQueryStringValue(HKLM, 'SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}', 'pv', v)
         or RegQueryStringValue(HKCU, 'Software\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}', 'pv', v);
  if Result then Result := (v <> '') and (v <> '0.0.0.0');
end;

function InitializeSetup: Boolean;
begin
  Result := True;
  if not WebView2Installed then
    Result := MsgBox('No se detectó Microsoft Edge WebView2 Runtime, necesario para DocFlow.' + #13#10 +
      'Instálelo desde microsoft.com/edge/webview2 y vuelva a ejecutar este instalador.' + #13#10#13#10 +
      '¿Continuar de todos modos?', mbConfirmation, MB_YESNO) = IDYES;
end;
