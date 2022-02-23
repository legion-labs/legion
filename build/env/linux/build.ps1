$ErrorActionPreference="SilentlyContinue"
Stop-Transcript | out-null
$ErrorActionPreference = "Stop"

cp $PSScriptRoot\..\..\..\rust-toolchain.toml $PSScriptRoot\install\rust-toolchain.toml
cp $PSScriptRoot\..\..\..\.monorepo\tools.toml $PSScriptRoot\install\tools.toml

docker build . -t build-env