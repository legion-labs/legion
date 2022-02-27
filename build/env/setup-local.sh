#!/bin/bash

# -e tells the shell to exit if a command exits with an error (except if the exit value is tested in some other way).
# -u tells the shell to treat expanding an unset parameter an error, which helps to catch e.g. typos in variable names.
# -x  tells the shell to print commands and their arguments as they are executed.
set -eux

script_dir="$(dirname "$0")"

pushd "$script_dir/install"

echo "------------------------------------ Install base utils ---------------------------------------"
./base.sh

echo "--------------------------------------- Build C++ --------------------------------------------"
./cpp.sh

echo "------------------------------------- Install Libs -------------------------------------------"
./libs.sh

echo "------------------------------------- Install Rust -------------------------------------------"
export RUSTUP_HOME=/usr/local/rustup
export CARGO_HOME=/usr/local/cargo

./rust.sh

export PATH="$CARGO_HOME/bin:$PATH"
export CC_x86_64_pc_windows_msvc="clang-cl"
export CXX_x86_64_pc_windows_msvc="clang-cl"
export AR_x86_64_pc_windows_msvc="llvm-lib"
export LDFLAGS="-fuse-ld=lld"
# Note that we only disable unused-command-line-argument here since clang-cl
# doesn't implement all of the options supported by cl, but the ones it doesn't
# are _generally_ not interesting.
export CL_FLAGS="-Wno-unused-command-line-argument -fuse-ld=lld-link /imsvc/xwin/crt/include /imsvc/xwin/sdk/include/ucrt /imsvc/xwin/sdk/include/um /imsvc/xwin/sdk/include/shared"
# Let cargo know what linker to invoke if you haven't already specified it
# in a .cargo/config.toml file
export CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER="lld-link"
export CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS="-Lnative=/xwin/crt/lib/x86_64 -Lnative=/xwin/sdk/lib/um/x86_64 -Lnative=/xwin/sdk/lib/ucrt/x86_64 -Lnative=/xwin/vulkan-sdk/Lib"

export CFLAGS_x86_64_pc_windows_msvc="$CL_FLAGS"
export CXXFLAGS_x86_64_pc_windows_msvc="$CL_FLAGS"

# Persist the environment variable for the profile.
echo "export RUSTUP_HOME=$RUSTUP_HOME
export CARGO_HOME=$CARGO_HOME
export PATH=$CARGO_HOME/bin:\$PATH
export CC_x86_64_pc_windows_msvc=$CC_x86_64_pc_windows_msvc
export CXX_x86_64_pc_windows_msvc=$CXX_x86_64_pc_windows_msvc
export AR_x86_64_pc_windows_msvc=$AR_x86_64_pc_windows_msvc
export LDFLAGS=\"$LDFLAGS\"
export CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER=$CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER
export CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS=\"$CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS\"
export CFLAGS_x86_64_pc_windows_msvc=\"$CL_FLAGS\"
export CXXFLAGS_x86_64_pc_windows_msvc=\"$CL_FLAGS\"" | tee /etc/profile.d/build.sh

echo "------------------------------------- Install Node ------------------------------------------"
export NODE_VERSION=16.14.0
export NVM_DIR=/usr/local/nvm
./node.sh

# This version is duplicated in the node.sh script, make sure to update both
echo "export NVM_DIR=/usr/local/nvm
export NODE_PATH=$NVM_DIR/v$NODE_VERSION/lib/node_modules
export PATH=$NVM_DIR/versions/node/v$NODE_VERSION/bin:\$PATH" | tee /etc/profile.d/build.sh

echo '------------------------------------ Cloud Tools -------------------------------------------------'
./cloud.sh
