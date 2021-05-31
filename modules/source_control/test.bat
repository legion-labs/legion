@echo off
SET LSC_BIN_DIR=%~dp0target\debug
rmdir /Q /S d:\temp\data
rmdir /Q /S d:\temp\workspace

cargo build
IF %ERRORLEVEL% NEQ 0 exit /b 1

%LSC_BIN_DIR%\lsc-cli.exe init-local-repository -r d:\temp\data
IF %ERRORLEVEL% NEQ 0 exit /b 1

%LSC_BIN_DIR%\lsc-cli.exe init-workspace -w d:\temp\workspace -r d:\temp\data
IF %ERRORLEVEL% NEQ 0 exit /b 1
