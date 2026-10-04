@echo off
REM Check that this repo can run the Makefile. Install GNU make, cargo, or
REM bash when they are missing. Invoked by the Makefile before other targets.
REM Run this file directly when `make` itself is not installed yet.
setlocal EnableDelayedExpansion

set "ROOT=%~dp0.."
set "FROM_MAKE=0"
set "NEED_BASH=0"
for %%A in (%*) do (
    if /I "%%~A"=="--from-make" set "FROM_MAKE=1"
    if /I "%%~A"=="sh" set "NEED_BASH=1"
)

where make >nul 2>&1
if errorlevel 1 call :install_make
if errorlevel 1 exit /b 1

where cargo >nul 2>&1
if errorlevel 1 call :install_rust
if errorlevel 1 exit /b 1

if "!NEED_BASH!"=="1" (
    where bash >nul 2>&1
    if errorlevel 1 call :install_bash
    if errorlevel 1 exit /b 1
)

REM Direct launch (not from make): hand off to make once it is available.
if "!FROM_MAKE!"=="0" (
    if exist "%ROOT%\.tools\bin\make.exe" set "PATH=%ROOT%\.tools\bin;%PATH%"
    where make >nul 2>&1
    if not errorlevel 1 (
        make -C "%ROOT%" %*
    )
)
exit /b 0

:install_make
echo GNU make is not installed. Installing it into .tools\bin ...
powershell -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command ^
  "$ErrorActionPreference='Stop'; $root=(Resolve-Path '%ROOT%').Path; $dest=Join-Path $root '.tools\bin'; New-Item -ItemType Directory -Force -Path $dest | Out-Null; $zip=Join-Path $env:TEMP 'devkit-make.nupkg.zip'; $extract=Join-Path $env:TEMP 'devkit-make-nupkg'; if (Test-Path $extract) { Remove-Item -Recurse -Force $extract }; Invoke-WebRequest -UseBasicParsing -Uri 'https://community.chocolatey.org/api/v2/package/make/4.4.1' -OutFile $zip; Expand-Archive -Force -Path $zip -DestinationPath $extract; Copy-Item -Force (Join-Path $extract 'tools\install\bin\*') $dest; if (-not (Test-Path (Join-Path $dest 'make.exe'))) { throw 'make.exe was not in the package' }"
if errorlevel 1 (
    echo Failed to install GNU make.
    echo Install it yourself, then run make again. For example: winget install GnuWin32.Make
    exit /b 1
)
echo GNU make installed at %ROOT%\.tools\bin\make.exe
echo Add that folder to PATH for new terminals.
set "PATH=%ROOT%\.tools\bin;%PATH%"
exit /b 0

:install_rust
echo cargo is not installed. Installing Rust ^(rustup, stable^)...
set "TRIPLE=x86_64-pc-windows-msvc"
if /I "%PROCESSOR_ARCHITECTURE%"=="ARM64" set "TRIPLE=aarch64-pc-windows-msvc"
mkdir "%TEMP%\devkit" >nul 2>nul
set "RUSTUP_INIT=%TEMP%\devkit\rustup-init.exe"
powershell -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command ^
  "Invoke-WebRequest -UseBasicParsing -Uri 'https://static.rust-lang.org/rustup/dist/%TRIPLE%/rustup-init.exe' -OutFile '%RUSTUP_INIT%'"
if not exist "%RUSTUP_INIT%" (
    echo Failed to download rustup-init.
    exit /b 1
)
"%RUSTUP_INIT%" -y --default-toolchain stable
if errorlevel 1 (
    echo rustup-init failed.
    exit /b 1
)
echo Rust was installed. Open a new terminal so cargo is on PATH, then run make again.
exit /b 1

:install_bash
echo bash is not installed. It is required for make sh.
where winget >nul 2>&1
if errorlevel 1 (
    echo Install Git for Windows so bash is on PATH, then run make sh again.
    echo   https://git-scm.com/download/win
    exit /b 1
)
winget install --id Git.Git -e --accept-package-agreements --accept-source-agreements
if errorlevel 1 (
    echo Failed to install Git for Windows.
    exit /b 1
)
echo Git was installed. Open a new terminal so bash is on PATH, then run make sh again.
exit /b 1
