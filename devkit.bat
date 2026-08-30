@echo off
setlocal
REM DevKit application launcher (Windows) — builds the release binary on first
REM run (or after source changes), then forwards to it.
set ROOT=%~dp0
set BIN=%ROOT%target\release\devkit.exe
if not exist "%BIN%" (
    cargo build --release --manifest-path "%ROOT%Cargo.toml" || exit /b 1
)
"%BIN%" %*
exit /b %ERRORLEVEL%
