@echo off
setlocal
REM DevKit application launcher (Windows)
python "%~dp0main.py" %*
exit /b %ERRORLEVEL%
