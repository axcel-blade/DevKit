@echo off
REM DevKit application launcher (Windows).
REM
REM 1. Use Rust from the machine `dev` folder (`C:\dev\rust`, or DEVKIT_HOME\rust).
REM    If cargo is missing there, require an internet connection and install
REM    rustup stable into that folder (same rustup-init flags as the rust
REM    plugin: -y --no-modify-path).
REM 2. Rebuild the release binary (a no-op when it is already up to date), so
REM    a stale binary from an older checkout never hides new features.
REM 3. Run it. With no arguments (e.g. double-clicking this file) the
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

REM Step 2: always run cargo build. Cargo only recompiles when sources
REM changed, so this is fast when up to date, and it guarantees the binary
REM matches this checkout (an old binary may predate the plugin menu).
"%CARGO_EXE%" build --release --quiet --manifest-path "%ROOT%Cargo.toml"
if errorlevel 1 (
    if not exist "%BIN%" (
        echo Build failed.
        goto :fail
    )
    echo Warning: build failed, running the previously built binary.
)

REM Step 3: no arguments means the user just launched DevKit (e.g. by
REM double-clicking), so open the interactive menu explicitly.
if "%~1"=="" (
    "%BIN%" menu
) else (
    "%BIN%" %*
)
exit /b %ERRORLEVEL%

REM Error exit. When launched without arguments (double-click) pause so the
REM console window stays open long enough to read the message.
:fail
if "%~1"=="" pause
exit /b 1
