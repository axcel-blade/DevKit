@echo off
REM DevKit application launcher (Windows).
REM
REM 1. Require an internet connection (needed to fetch rustup / crates).
REM 2. Use Rust from the machine `dev` folder (`C:\dev\rust`, or DEVKIT_HOME\rust).
REM    If cargo is missing there, install rustup stable into that folder
REM    (same rustup-init flags as the rust plugin: -y --no-modify-path).
REM 3. Build the release binary if needed, then run it.
setlocal EnableDelayedExpansion

set "ROOT=%~dp0"
set "BIN=%ROOT%target\release\devkit.exe"

REM Step 1: fail fast if we cannot reach rustup's CDN (HTTPS/443).
echo Checking internet connection...
powershell -NoProfile -NonInteractive -Command ^
    "try { $c = New-Object Net.Sockets.TcpClient; $c.Connect('static.rust-lang.org', 443); $c.Close() } catch { exit 1 }"
if errorlevel 1 (
    echo Error: no internet connection. Connect to the internet and try again.
    exit /b 1
)

REM Step 2: same machine `dev` root as paths::home() (DEVKIT_HOME or C:\dev).
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
    echo Installing Rust ^(rustup, stable^) into the machine dev folder...

    set "TRIPLE=x86_64-pc-windows-msvc"
    if /I "%PROCESSOR_ARCHITECTURE%"=="ARM64" set "TRIPLE=aarch64-pc-windows-msvc"

    mkdir "%DEVROOT%\rust" >nul 2>nul
    set "RUSTUP_INIT=%TEMP%\devkit-rustup-init.exe"
    if exist "!RUSTUP_INIT!" del /f /q "!RUSTUP_INIT!" >nul 2>nul
    powershell -NoProfile -NonInteractive -Command ^
        "Invoke-WebRequest -UseBasicParsing -Uri 'https://static.rust-lang.org/rustup/dist/!TRIPLE!/rustup-init.exe' -OutFile '!RUSTUP_INIT!'"
    if not exist "!RUSTUP_INIT!" (
        echo Failed to download rustup-init. Check your internet connection and try again.
        exit /b 1
    )

    set "CARGO_HOME=%DEVROOT%\rust\cargo"
    set "RUSTUP_HOME=%DEVROOT%\rust\rustup"
    "!RUSTUP_INIT!" -y --no-modify-path --default-toolchain stable --profile default
    if errorlevel 1 (
        echo rustup-init failed.
        exit /b 1
    )
    del /f /q "!RUSTUP_INIT!" >nul 2>nul

    if not exist "%CARGO_EXE%" (
        echo Rust install finished but cargo.exe was not found at %CARGO_EXE%.
        exit /b 1
    )
    echo Rust installed at %DEVROOT%\rust.
)

if not exist "%BIN%" (
    "%CARGO_EXE%" build --release --manifest-path "%ROOT%Cargo.toml" || exit /b 1
)
"%BIN%" %*
exit /b %ERRORLEVEL%
