#!/bin/bash

# -e tells the shell to exit if a command exits with an error (except if the exit value is tested in
#    some other way).
# -u tells the shell to treat expanding an unset parameter an error, which helps to catch e.g. typos
#    in variable names.
# -x  tells the shell to print commands and their arguments as they are executed.
set -eux

###################################################################################################

LLVM_VERSION=13
XWIN_VERSION=0.1.6
NINJA_VERSION=1.10.2
CMAKE_VERSION=3.22.2

###################################################################################################

source "$(dirname "$0")/helper.sh"

###################################################################################################

# Install base build utils
apt-get update && apt-get install -y \
    pkg-config \
    build-essential \
    musl-tools \
    nasm

###################################################################################################

# Clang and LLVM
echo $DISTRO_NAME_VERSION
case "$DISTRO_NAME_VERSION" in
    Debian_11* )     REPO_NAME="deb http://apt.llvm.org/bullseye/  llvm-toolchain-bullseye-$LLVM_VERSION  main" ;;
    Ubuntu_20.04 )   REPO_NAME="deb http://apt.llvm.org/focal/     llvm-toolchain-focal-$LLVM_VERSION   main" ;;
    Ubuntu_22.04 )   REPO_NAME="deb http://apt.llvm.org/jammy/     llvm-toolchain-jammy-$LLVM_VERSION main" ;;
    * )
        echo "Distribution '$DISTRO' in version '$DISTRO_VERSION' is not supported by this script (${DISTRO_NAME_VERSION})."
        exit 1
esac

wget -O - https://apt.llvm.org/llvm-snapshot.gpg.key | apt-key add -
add-apt-repository "${REPO_NAME}"
apt-get update && apt-get install -y \
    clang-$LLVM_VERSION llvm-$LLVM_VERSION lldb-$LLVM_VERSION lld-$LLVM_VERSION \
    clang-format-$LLVM_VERSION libclang-$LLVM_VERSION-dev libunwind-$LLVM_VERSION-dev \
    llvm-$LLVM_VERSION-tools


# link main clang executables
ln -sf clang-$LLVM_VERSION /usr/bin/clang
ln -sf clang /usr/bin/clang++
ln -sf lld-$LLVM_VERSION /usr/bin/ld.lld
ln -sf clang-format-${LLVM_VERSION} /usr/bin/clang-format

# MSVC links
ln -sf clang-$LLVM_VERSION /usr/bin/clang-cl
ln -sf llvm-ar-$LLVM_VERSION /usr/bin/llvm-lib
ln -sf lld-link-$LLVM_VERSION /usr/bin/lld-link
ln -sf llvm-ml-$LLVM_VERSION /usr/bin/ml64.exe

# Use clang instead of gcc when compiling binaries targeting the host (eg proc macros, build files)
update-alternatives --install /usr/bin/cc cc /usr/bin/clang 100
update-alternatives --install /usr/bin/c++ c++ /usr/bin/clang++ 100

###################################################################################################

# Win SDK
XWIN_PREFIX="xwin-$XWIN_VERSION-x86_64-unknown-linux-musl"
wget -qO- https://github.com/Jake-Shadle/xwin/releases/download/$XWIN_VERSION/$XWIN_PREFIX.tar.gz |
    tar -xzv -C /usr/bin --strip-components=1 $XWIN_PREFIX/xwin
xwin --accept-license 1 --version 17 --cache-dir /tmp/xwin-cache splat --output /xwin
ln -sf /xwin/sdk/lib/um/x86_64/iphlpapi.lib /xwin/sdk/lib/um/x86_64/Iphlpapi.lib
rm -rf /tmp/xwin-cache

###################################################################################################

# MacOS for down the road maybe?
# https://github.com/tpoechtrager/osxcross

###################################################################################################

# Ninja
wget -qO ninja-linux.zip https://github.com/ninja-build/ninja/releases/download/v$NINJA_VERSION/ninja-linux.zip
unzip ninja-linux.zip
mv ninja /usr/bin/ninja
rm ninja-linux.zip
chmod a+x /usr/bin/ninja

###################################################################################################

# CMake
CMAKE_PREFIX="cmake-$CMAKE_VERSION-linux-x86_64"
wget -qO- https://github.com/Kitware/CMake/releases/download/v$CMAKE_VERSION/$CMAKE_PREFIX.tar.gz |
    tar -xzC /opt

ln -sf /opt/$CMAKE_PREFIX/bin/cmake /usr/bin/cmake
