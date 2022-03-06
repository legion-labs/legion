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
scoop install git zip unzip python jq
scoop bucket add extras
scoop bucket add legion-labs https://github.com/legion-labs/scoop-bucket
scoop update

Write-Host $(date) '--------------------- Install C++ -------------------------------------'
scoop install legion-labs/llvm
scoop install cmake
scoop install ninja
scoop install nasm

Write-Host $(date) '--------------------- Install Rust -----------------------------------'
scoop install rustup-msvc

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

Stop-Transcript
