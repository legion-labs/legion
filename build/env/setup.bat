@echo off

set SCRIPT_DIR=%~dp0
set SCRIPT_DIR=%SCRIPT_DIR:~0,-1%
set MONOREPO_ROOT="%SCRIPT_DIR%\..\.."

@REM These files are outide the context
copy "%MONOREPO_ROOT%\rust-toolchain.toml" "%SCRIPT_DIR%\install\rust-toolchain.toml" 1> nul
copy "%MONOREPO_ROOT%\.monorepo\tools.toml" "%SCRIPT_DIR%\install\tools.toml" 1> nul

pushd %SCRIPT_DIR%

set BB=.\utils\busybox.exe
for /f %%i in ('%BB% sha1sum install/* ^| %BB% sha1sum ^| %BB% head -c 40') do (
    set CONTAINER_HASH=%%i
)

set TAG="build-env:%CONTAINER_HASH%"
for /f %%i in ('docker images -q %TAG% ^2^> nul') do (
    set LOCAL_CONTAINER_EXISTS="1"
)
if defined MONOREPO_DOCKER_REGISTRY (
    set REPO_TAG="%MONOREPO_DOCKER_REGISTRY%/%TAG%"
    aws ecr get-login-password --region ca-central-1 | docker login --username AWS --password-stdin
    docker pull %REPO_TAG%
    if "%ERRORLEVEL%"=="0" (
        docker build . -t %TAG%
        docker tag %REPO_TAG%
        docker push %REPO_TAG%
    )
) else (
    if "%LOCAL_CONTAINER_EXISTS%"=="" (
        docker build . -t %TAG%
    ) else (
        echo "Image %TAG% already exists"
    )
)

del install\rust-toolchain.toml install\tools.toml

popd