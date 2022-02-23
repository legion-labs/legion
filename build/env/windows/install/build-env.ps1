$ErrorActionPreference="SilentlyContinue"
Stop-Transcript | out-null
$ErrorActionPreference = "Stop"

$working_dir = "c:\tmp"
New-Item -Path $working_dir -ItemType directory
cd $working_dir
$todaytime = Get-Date -UFormat '%Y%m%d%H%M'
Start-Transcript -path $working_dir"\$todaytime"_output.txt -append
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$env:SCOOP = 'C:\Scoop\user'
$env:SCOOP_GLOBAL = 'C:\Scoop\global'

Write-Host $(date) '--------------------- Update scoop ------------------------------------'
[System.Environment]::SetEnvironmentVariable('SCOOP','C:\Scoop\user', 'Machine')
[System.Environment]::SetEnvironmentVariable('SCOOP','C:\Scoop\user', 'User')

[System.Environment]::SetEnvironmentVariable('SCOOP_GLOBAL','C:\Scoop\global', 'Machine')
[System.Environment]::SetEnvironmentVariable('SCOOP_GLOBAL','C:\Scoop\global', 'User')

Invoke-Expression (New-Object System.Net.WebClient).DownloadString('https://get.scoop.sh')
scoop install git zip unzip make python jq
scoop bucket add extras
scoop bucket add legion-labs https://github.com/legion-labs/scoop-bucket
scoop update

Write-Host $(date) '--------------------- Install C++ -------------------------------------'
scoop install winsdk --global
scoop install vs_buildtools --global
scoop install legion-labs/llvm
scoop install cmake
scoop install ninja
scoop install nasm
scoop cache rm llvm

Write-Host $(date) '--------------------- Install Rust -----------------------------------'
scoop install rustup-msvc
rustup install 1.58.0
rustup install 1.58.1
rustup target add x86_64-unknown-linux-musl --toolchain '1.58.0'
rustup target add x86_64-unknown-linux-musl --toolchain '1.58.1'
cargo install cargo-deny --version "0.11.1" --locked
cargo install mdbook --version "0.4.15" --locked
cargo install sccache --git "https://github.com/diem/sccache.git" --rev ef50d87a58260c30767520045e242ccdbdb965af
cargo install wasm-bindgen-cli --version "0.2.79"
Remove-Item "$env:SCOOP\persist\rustup-msvc\.cargo\registry" -Force -Recurse
Remove-Item "$env:SCOOP\persist\rustup-msvc\.cargo\git" -Force -Recurse

Write-Host $(date) '---------------------- Install VulkanSdk/dcx ---------------------------'
scoop install legion-labs/vulkan --global

Write-Host $(date) '------------------- Install AWS Sdk and Online tools -------------------'
scoop install terraform
scoop install kubectl
scoop install aws
Copy-Item "$env:SCOOP\apps\aws\current\awscli\botocore\cacert.pem" "$env:SCOOP\apps\aws\current\awscli\certifi"

Write-Host $(date) '------------------- Install NodeJS and Web tools -------------------'
scoop install nvm
scoop install protobuf
nvm install 16.10.0
nvm use 16.10.0
npm --version
npm -g i yarn pnpm

Write-Host $(date) '-----------------------------------------------------------------------'
Write-Host $(date) '-------------------------- Perform Checks -----------------------------'
Write-Host $(date) '-----------------------------------------------------------------------'

Write-Host $(date) '------------------------ Check base utils -----------------------------'
scoop --version
git --version
python --version

Write-Host $(date) '------------------------- Check cpp utils -----------------------------'
ninja --version
cmake --version
nasm --version

git clone https://github.com/jameskbride/cmake-hello-world.git

Push-Location cmake-hello-world
    New-Item build -ItemType Directory
    Push-Location build
        cmake ..
        cmake --build .
    Pop-Location
Pop-Location
Remove-Item cmake-hello-world -Recurse -Force

Write-Host $(date) '------------------------- Check rust utils -----------------------------'
rustup show
cargo new test_rust
Push-Location test_rust
    cargo build
    cargo run
Pop-Location
Remove-Item test_rust -Recurse -Force

Stop-Transcript
