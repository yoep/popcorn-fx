[Setup]
AppId=Popcorn-Time
AppName=Popcorn Time
AppVersion=0.8.1
AppVerName=Popcorn-Time 0.8.1
AppPublisher=Popcorn FX
AppPublisherURL=https://github.com/yoep/popcorn-fx
AppSupportURL=https://github.com/yoep/popcorn-fx
AppUpdatesURL=https://github.com/yoep/popcorn-fx/releases
DefaultDirName={autopf}\popcorn-time
DisableDirPage=no
DefaultGroupName=Popcorn Time
DisableProgramGroupPage=no
DisableFinishedPage=no
DisableWelcomePage=no
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=commandline
LicenseFile=../../LICENSE
SetupIconFile=./popcorn-time.ico
UninstallDisplayIcon={app}\popcorn-time.exe
Compression=lzma
SolidCompression=yes
ArchitecturesInstallIn64BitMode=x64

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "dutch"; MessagesFile: "compiler:Languages\Dutch.isl"
Name: "french"; MessagesFile: "compiler:Languages\French.isl"
Name: "german"; MessagesFile: "compiler:Languages\German.isl"
Name: "spanish"; MessagesFile: "compiler:Languages\Spanish.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Registry]

[Files]
Source: "../../target/package/popcorn-time.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "../../target/package/runtimes/*"; DestDir: "{app}\main\runtimes"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "../../target/package/popcorn-time.jar"; DestDir: "{app}\main\0.8.1"; Flags: ignoreversion
Source: "../../target/package/popcorn_fx.dll"; DestDir: "{app}\main\0.8.1"; Flags: ignoreversion
Source: "popcorn-time.ico"; DestDir: "{app}"; Flags: ignoreversion
Source: "jlibtorrent.dll"; DestDir: "{app}\main\0.8.1"; Flags: ignoreversion
Source: "ffprobe.exe"; DestDir: "{app}\main\0.8.1"; Flags: ignoreversion

[UninstallDelete]
Type: files; Name: "{userappdata}\popcorn-fx"

[Icons]
Name: "{autoprograms}\popcorn-time"; Filename: "{app}\popcorn-time.exe"; IconFilename: "{app}\\popcorn-time.ico"
Name: "{autodesktop}\popcorn-time"; Filename: "{app}\popcorn-time.exe"; IconFilename: "{app}\\popcorn-time.ico"; Tasks: desktopicon

[Run]


[Code]

function GetInstallLocation(): String;
var
    unInstPath: String;
    installLocation: String;
begin
    unInstPath := ExpandConstant('Software\Microsoft\Windows\CurrentVersion\Uninstall\{#emit SetupSetting("AppId")}_is1');
    installLocation := '';
    if not RegQueryStringValue(HKLM, unInstPath, 'InstallLocation', installLocation) then
        RegQueryStringValue(HKCU, unInstPath, 'InstallLocation', installLocation);
    Result := RemoveQuotes(installLocation);
end;

procedure RemoveOldLibs();
var
    installLocation: String;
    libsLocation: String;
begin
    installLocation := GetInstallLocation();
    if installLocation <> '' then
    begin
        libsLocation := installLocation + 'libs';
        DelTree(libsLocation, True, True, True);
    end;
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
    if CurStep = ssInstall then
    begin
        RemoveOldLibs();
    end;
end;