@echo off

set SCRIPT_DIR=%~dp0
set SCRIPT_DIR=%SCRIPT_DIR:~0,-1%
set MONOREPO_ROOT="%SCRIPT_DIR%\..\.."

@REM These files are outide the context
copy "%MONOREPO_ROOT%\rust-toolchain.toml" "%SCRIPT_DIR%\install\rust-toolchain.toml" 1> nul
copy "%MONOREPO_ROOT%\.monorepo\tools.toml" "%SCRIPT_DIR%\install\tools.toml" 1> nul

pushd %SCRIPT_DIR%

set IMAGE_NAME="build-env"

set BB=.\utils\busybox.exe
for /f %%i in ('%BB% sha1sum Dockerfile install/* ^| %BB% sha1sum ^| %BB% head -c 40') do (
    set IMAGE_TAG=%%i
)

set IMAGE="%IMAGE_NAME%:%IMAGE_TAG%"
for /f %%i in ('docker images -q %IMAGE% ^2^> nul') do (
    set LOCAL_CONTAINER_EXISTS="1"
)
if defined MONOREPO_DOCKER_REGISTRY (
    set IMAGE="%MONOREPO_DOCKER_REGISTRY%/%IMAGE_NAME%:%IMAGE_TAG%"
    aws ecr get-login-password --region ca-central-1 | docker login --username AWS --password-stdin
    docker manifest inspect $IMAGE > nul 2>&1
    if "%ERRORLEVEL%"=="0" (
        docker build . -t %IMAGE%
        docker push %IMAGE%
    )
) else (
    if "%LOCAL_CONTAINER_EXISTS%"=="" (
        docker build . -t %IMAGE%
    ) else (
        echo "Image %IMAGE% already exists"
    )
)

del install\rust-toolchain.toml install\tools.toml

popd