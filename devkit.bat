@echo off
REM DevKit application launcher (Windows).
REM
REM DevKit itself is built with Rust, so before anything else this makes
REM sure a Rust toolchain is available: if `cargo` isn't found, it checks
REM for internet access, then downloads and runs rustup-init the same way
REM the built-in `rust` plugin does (same host-triple URL, `-y --default-
REM toolchain stable --profile default`) — except into the *standard*
REM ~/.cargo location, since this Rust install is what builds DevKit, not
REM an SDK DevKit is managing for someone else.
REM
REM Once cargo is available, it builds the release binary on first run (or
REM after source changes) and forwards all arguments to it. Running with no
REM arguments launches the interactive plugin menu (see `devkit menu`).
setlocal EnableDelayedExpansion

set "ROOT=%~dp0"
set "BIN=%ROOT%target\release\devkit.exe"
set "CARGO_EXE="

where cargo >nul 2>nul
if %ERRORLEVEL% EQU 0 set "CARGO_EXE=cargo"
if not defined CARGO_EXE if exist "%USERPROFILE%\.cargo\bin\cargo.exe" (
    set "CARGO_EXE=%USERPROFILE%\.cargo\bin\cargo.exe"
)

if not defined CARGO_EXE (
    echo DevKit needs a Rust toolchain ^(cargo^) to build itself — none found.
    echo Checking internet connection...

    powershell -NoProfile -NonInteractive -Command ^
        "try { $c = New-Object Net.Sockets.TcpClient; $c.Connect('static.rust-lang.org', 443); $c.Close() } catch { exit 1 }"
    if errorlevel 1 (
        echo No internet connection detected.
        echo Install Rust manually from https://rustup.rs and re-run this script.
        exit /b 1
    )

    set "TRIPLE=x86_64-pc-windows-msvc"
    if /I "%PROCESSOR_ARCHITECTURE%"=="ARM64" set "TRIPLE=aarch64-pc-windows-msvc"

    echo Installing Rust ^(rustup, stable^) for !TRIPLE! ...
    set "RUSTUP_INIT=%TEMP%\devkit-rustup-init.exe"
    if exist "!RUSTUP_INIT!" del /f /q "!RUSTUP_INIT!" >nul 2>nul
    powershell -NoProfile -NonInteractive -Command ^
        "Invoke-WebRequest -UseBasicParsing -Uri 'https://static.rust-lang.org/rustup/dist/!TRIPLE!/rustup-init.exe' -OutFile '!RUSTUP_INIT!'"
    if not exist "!RUSTUP_INIT!" (
        echo Failed to download rustup-init.
        exit /b 1
    )

    "!RUSTUP_INIT!" -y --default-toolchain stable --profile default
    if errorlevel 1 (
        echo rustup-init failed.
        exit /b 1
    )
    del /f /q "!RUSTUP_INIT!" >nul 2>nul

    set "CARGO_EXE=%USERPROFILE%\.cargo\bin\cargo.exe"
    if not exist "!CARGO_EXE!" (
        echo Rust install finished but cargo.exe was not found at !CARGO_EXE!.
        exit /b 1
    )
    echo Rust installed. Future terminals will have 'cargo' on PATH automatically.
)

if not exist "%BIN%" (
    "%CARGO_EXE%" build --release --manifest-path "%ROOT%Cargo.toml" || exit /b 1
)
"%BIN%" %*
exit /b %ERRORLEVEL%
