@echo off
REM DevKit application launcher (Windows).
REM
REM 1. Use Rust from the machine `dev` folder (`C:\dev\rust`, or DEVKIT_HOME\rust).
REM    If cargo is missing there, require an internet connection and install
REM    rustup stable into that folder (same rustup-init flags as the rust
REM    plugin: -y --no-modify-path).
REM 2. Make sure the MSVC linker (link.exe from Visual Studio Build Tools with
REM    the "Desktop development with C++" workload) is present. The msvc Rust
REM    toolchain cannot link anything without it, and several dependencies
REM    (ring, zstd-sys, lzma-sys, bzip2-sys) also need its C compiler. When
REM    missing, first add the C++ tools to an existing Visual Studio install
REM    (via its setup.exe modify); otherwise install the Build Tools through
REM    winget, or through Microsoft's vs_BuildTools.exe bootstrapper when
REM    winget is unavailable or fails.
REM 3. Rebuild the release binary (a no-op when it is already up to date), so
REM    a stale binary from an older checkout never hides new features.
REM 4. Run it. With no arguments (e.g. double-clicking this file) the
REM    interactive plugin menu is shown.
setlocal EnableDelayedExpansion

set "ROOT=%~dp0"
set "BIN=%ROOT%target\release\devkit.exe"

REM Step 1: same machine `dev` root as paths::home() (DEVKIT_HOME or C:\dev).
if defined DEVKIT_HOME (
    set "DEVROOT=%DEVKIT_HOME%"
) else (
    if not defined SystemDrive set "SystemDrive=C:"
    set "DEVROOT=%SystemDrive%\dev"
)

set "CARGO_HOME=%DEVROOT%\rust\cargo"
set "RUSTUP_HOME=%DEVROOT%\rust\rustup"
set "CARGO_EXE=%CARGO_HOME%\bin\cargo.exe"

if not exist "%CARGO_EXE%" (
    echo Rust is not installed in %DEVROOT%\rust.

    REM Installing Rust needs rustup's CDN, so fail fast when it is
    REM unreachable. An already-installed toolchain skips this check so
    REM the menu still opens offline.
    echo Checking internet connection...
    powershell -NoProfile -NonInteractive -Command ^
        "try { $c = New-Object Net.Sockets.TcpClient; $c.Connect('static.rust-lang.org', 443); $c.Close() } catch { exit 1 }"
    if errorlevel 1 (
        echo Error: no internet connection. Connect to the internet and try again.
        goto :fail
    )

    echo Installing Rust ^(rustup, stable^) into the machine dev folder...

    set "TRIPLE=x86_64-pc-windows-msvc"
    if /I "%PROCESSOR_ARCHITECTURE%"=="ARM64" set "TRIPLE=aarch64-pc-windows-msvc"

    mkdir "%DEVROOT%\rust" >nul 2>nul
    REM rustup picks its mode from its own file name, so the installer must
    REM be named exactly rustup-init.exe (a prefixed name such as
    REM devkit-rustup-init.exe fails with "unknown proxy name"). Keep it in
    REM a DevKit-specific temp folder instead.
    mkdir "%TEMP%\devkit" >nul 2>nul
    set "RUSTUP_INIT=%TEMP%\devkit\rustup-init.exe"
    if exist "!RUSTUP_INIT!" del /f /q "!RUSTUP_INIT!" >nul 2>nul
    powershell -NoProfile -NonInteractive -Command ^
        "Invoke-WebRequest -UseBasicParsing -Uri 'https://static.rust-lang.org/rustup/dist/!TRIPLE!/rustup-init.exe' -OutFile '!RUSTUP_INIT!'"
    if not exist "!RUSTUP_INIT!" (
        echo Failed to download rustup-init. Check your internet connection and try again.
        goto :fail
    )

    set "CARGO_HOME=%DEVROOT%\rust\cargo"
    set "RUSTUP_HOME=%DEVROOT%\rust\rustup"
    "!RUSTUP_INIT!" -y --no-modify-path --default-toolchain stable --profile default
    if errorlevel 1 (
        echo rustup-init failed.
        goto :fail
    )
    del /f /q "!RUSTUP_INIT!" >nul 2>nul

    if not exist "%CARGO_EXE%" (
        echo Rust install finished but cargo.exe was not found at %CARGO_EXE%.
        goto :fail
    )
    echo Rust installed at %DEVROOT%\rust.
)

set "BUILD_LOG=%TEMP%\devkit-build.log"
if exist "%BUILD_LOG%" del /f /q "%BUILD_LOG%" >nul 2>&1

REM Step 2: the msvc toolchain needs link.exe from the Visual C++ Build Tools.
call :ensure_msvc
if errorlevel 1 goto :build_failed

REM Step 3: always run cargo build. Cargo only recompiles when sources
REM changed, so this is fast when up to date, and it guarantees the binary
REM matches this checkout (an old binary may predate the plugin menu).
REM
REM Windows cannot delete a running executable (Access denied, os error 5).
REM A previous DevKit window keeps target\release\devkit.exe locked. Rename
REM that image aside — a running exe can be renamed — then link a new one.
del /f /q "%ROOT%target\release\devkit-*.old" >nul 2>&1
call :cargo_build
if not errorlevel 1 goto :run

findstr /C:"os error 5" "%BUILD_LOG%" >nul
if errorlevel 1 goto :build_failed

set "OLDNAME=devkit-!RANDOM!.old"
ren "%BIN%" "!OLDNAME!"
if errorlevel 1 goto :build_failed

echo A previous DevKit is still running. Rebuilding beside it...
call :cargo_build
if not errorlevel 1 goto :cleanup_old
if not exist "%BIN%" ren "%ROOT%target\release\!OLDNAME!" "devkit.exe"
goto :build_failed

:cleanup_old
del /f /q "%ROOT%target\release\devkit-*.old" >nul 2>&1
goto :run

:build_failed
if exist "%BUILD_LOG%" type "%BUILD_LOG%"
if not exist "%BIN%" (
    echo Build failed.
    goto :fail
)
echo Warning: build failed, running the previously built binary.

:run
REM Step 4: no arguments means the user just launched DevKit (e.g. by
REM double-clicking), so open the interactive menu explicitly.
if "%~1"=="" (
    "%BIN%" menu
) else (
    "%BIN%" %*
)
exit /b %ERRORLEVEL%

REM Succeeds when the Visual C++ toolset (and so link.exe) is installed.
REM vswhere ships with every Visual Studio / Build Tools installer and is how
REM rustc itself locates link.exe, so a VS install that lacks the C++ tools
REM component counts as missing. A link.exe already on PATH (e.g. inside a
REM Developer Command Prompt) is accepted too.
:has_msvc
set "VSWHERE=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe"
set "VSPATH="
if exist "%VSWHERE%" (
    for /f "usebackq delims=" %%I in (`"%VSWHERE%" -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath`) do set "VSPATH=%%I"
)
if defined VSPATH exit /b 0
where link.exe >nul 2>&1
exit /b %ERRORLEVEL%

REM Installs Visual Studio Build Tools with the VC++ workload when missing.
REM The installer asks for administrator rights (UAC) and takes a few minutes.
:ensure_msvc
call :has_msvc
if not errorlevel 1 exit /b 0

echo The Microsoft C++ linker ^(link.exe^) was not found.
echo Rust's msvc toolchain needs Visual Studio Build Tools with the C++ workload.
echo Installing Visual Studio Build Tools ^(several minutes, asks for admin rights^)...
REM The C++ tools plus Windows SDK need roughly 7 GB on the system drive
REM (packages are cached there even when VS lives elsewhere). Warn early.
call :check_disk_space
if errorlevel 1 exit /b 1
set "VS_ARGS=--quiet --wait --norestart --nocache --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"

REM An existing Visual Studio (Community, Professional, Build Tools, ...)
REM that only lacks the C++ tools: modify it through its own installer
REM instead of installing a second product next to it, which the VS
REM installer rejects.
REM
REM Exit code 0x80070070 (-2147024784 / 2147942512) is ERROR_DISK_FULL: the
REM installer's SizePreCheckEvaluator found too little free space. Retrying
REM with Build Tools would fail the same way, so stop with a clear message.
set "VS_SETUP=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\setup.exe"
set "VS_EXISTING="
if exist "%VSWHERE%" (
    for /f "usebackq delims=" %%I in (`"%VSWHERE%" -latest -products * -property installationPath`) do set "VS_EXISTING=%%I"
)
if defined VS_EXISTING if exist "%VS_SETUP%" (
    echo Adding the C++ tools to the existing Visual Studio at "!VS_EXISTING!"...
    REM Start-Process -Verb RunAs raises the UAC prompt; -Wait blocks until done.
    powershell -NoProfile -NonInteractive -Command ^
        "$q = [char]34; $a = 'modify --installPath ' + $q + $env:VS_EXISTING + $q + ' --add Microsoft.VisualStudio.Component.VC.Tools.x86.x64 --add Microsoft.VisualStudio.Component.Windows11SDK.22621 --includeRecommended --passive --norestart'; $p = Start-Process -FilePath $env:VS_SETUP -Verb RunAs -Wait -PassThru -ArgumentList $a; exit $p.ExitCode"
    set "VS_RC=!ERRORLEVEL!"
    if "!VS_RC!"=="-2147024784" goto :msvc_disk_full
    call :has_msvc
    if not errorlevel 1 (
        echo Visual Studio C++ tools installed.
        exit /b 0
    )
    echo Modifying the existing Visual Studio did not add the C++ tools; trying Build Tools...
)

set "WINGET_OK="
where winget >nul 2>&1
if not errorlevel 1 (
    winget install --id Microsoft.VisualStudio.2022.BuildTools -e --source winget --accept-package-agreements --accept-source-agreements --override "!VS_ARGS!"
    set "VS_RC=!ERRORLEVEL!"
    if "!VS_RC!"=="0" set "WINGET_OK=1"
    REM winget surfaces the installer's disk-full code (0x80070070).
    if "!VS_RC!"=="-2147024784" goto :msvc_disk_full
)
REM No winget, or winget failed: fall back to Microsoft's Build Tools bootstrapper.
call :has_msvc
if errorlevel 1 if not defined WINGET_OK (
    mkdir "%TEMP%\devkit" >nul 2>nul
    set "VS_BOOT=%TEMP%\devkit\vs_BuildTools.exe"
    powershell -NoProfile -NonInteractive -Command ^
        "Invoke-WebRequest -UseBasicParsing -Uri 'https://aka.ms/vs/17/release/vs_BuildTools.exe' -OutFile '!VS_BOOT!'"
    if exist "!VS_BOOT!" (
        "!VS_BOOT!" !VS_ARGS!
        del /f /q "!VS_BOOT!" >nul 2>nul
    ) else (
        echo Failed to download the Visual Studio Build Tools installer.
    )
)

call :has_msvc
if not errorlevel 1 (
    echo Visual Studio Build Tools installed.
    exit /b 0
)
echo Error: the C++ Build Tools are still missing.
echo Install "Build Tools for Visual Studio" from https://visualstudio.microsoft.com/visual-cpp-build-tools/
echo with the "Desktop development with C++" workload, then run DevKit again.
exit /b 1

REM Installer reported ERROR_DISK_FULL (0x80070070).
:msvc_disk_full
echo.
echo Error: not enough free disk space to install the C++ Build Tools ^(0x80070070^).
echo The Visual Studio installer needs about 7 GB free on %SystemDrive%.
echo Free up space ^(e.g. Disk Cleanup, empty the Recycle Bin, clear %TEMP%^), then run DevKit again.
exit /b 1

REM Fails when the system drive has less than ~7 GB free, since the VS
REM installer caches packages there and its pre-check rejects low space.
:check_disk_space
set "FREE_GB="
for /f "usebackq delims=" %%F in (`powershell -NoProfile -NonInteractive -Command "[math]::Floor((Get-PSDrive -Name $env:SystemDrive.TrimEnd(':')).Free / 1GB)"`) do set "FREE_GB=%%F"
if not defined FREE_GB exit /b 0
if !FREE_GB! LSS 7 (
    echo Only !FREE_GB! GB free on %SystemDrive%; the C++ Build Tools need about 7 GB.
    goto :msvc_disk_full
)
exit /b 0

:cargo_build
"%CARGO_EXE%" build --release --quiet --manifest-path "%ROOT%Cargo.toml" > "%BUILD_LOG%" 2>&1
exit /b %ERRORLEVEL%

REM Error exit. When launched without arguments (double-click) pause so the
REM console window stays open long enough to read the message.
:fail
if "%~1"=="" pause
exit /b 1
