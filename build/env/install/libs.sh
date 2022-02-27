#!/bin/bash

# -e tells the shell to exit if a command exits with an error (except if the exit value is tested in
#    some other way).
# -u tells the shell to treat expanding an unset parameter an error, which helps to catch e.g. typos
#    in variable names.
# -x  tells the shell to print commands and their arguments as they are executed.
set -eux

###################################################################################################

VULKAN_VERSION=1.3.204
DXC_VERSION=1.6.2112

###################################################################################################

dpkg --add-architecture i386

wget -O - https://dl.winehq.org/wine-builds/winehq.key | apt-key add -

source "$(dirname "$0")/helper.sh"

case "$DISTRO_NAME_VERSION" in
    Debian_11* )     add-apt-repository "deb https://dl.winehq.org/wine-builds/debian/ bullseye  main" ;;
    Ubuntu_20.04 )   add-apt-repository "deb https://dl.winehq.org/wine-builds/ubuntu/ focal main" ;;
    Ubuntu_22.04 )   add-apt-repository "deb https://dl.winehq.org/wine-builds/ubuntu/ jammy main" ;;
    * )
        echo "Distribution '$DISTRO' in version '$DISTRO_VERSION' is not supported by this script (${DISTRO_NAME_VERSION})."
        exit 1
esac

wget -qO - https://packages.lunarg.com/lunarg-signing-key-pub.asc | apt-key add -
case "$DISTRO_NAME_VERSION" in
    Debian_11* )
        wget -qO /etc/apt/sources.list.d/lunarg-vulkan-$VULKAN_VERSION-bullseye.list \
            https://packages.lunarg.com/vulkan/$VULKAN_VERSION/lunarg-vulkan-$VULKAN_VERSION-bullseye.list
        ;;
    Ubuntu_20.04 )
        wget -qO /etc/apt/sources.list.d/lunarg-vulkan-$VULKAN_VERSION-focal.list \
            https://packages.lunarg.com/vulkan/$VULKAN_VERSION/lunarg-vulkan-$VULKAN_VERSION-focal.list
        ;;
    Ubuntu_22.04 )
        wget -qO /etc/apt/sources.list.d/lunarg-vulkan-$VULKAN_VERSION-jammy.list \
            https://packages.lunarg.com/vulkan/$VULKAN_VERSION/lunarg-vulkan-$VULKAN_VERSION-jammy.list
        ;;
    * )
        echo "Distribution '$DISTRO' in version '$DISTRO_VERSION' is not supported by this script (${DISTRO_NAME_VERSION})."
        exit 1
esac

apt-get update && apt-get install -y --no-install-recommends \
    libssl-dev \
    libglib2.0-dev \
    libcairo-dev \
    librust-pango-dev \
    libatk1.0-dev \
    libsoup2.4-dev \
    libgdk-pixbuf2.0-dev \
    librust-gdk-sys-dev \
    libwebkit2gtk-4.0-dev \
    fuse3 \
    libfuse3-dev \
    vulkan-sdk

# We are pinning wine to 6.23 since tokio/mio port binding fails
apt-get install -y --no-install-recommends \
    winehq-staging=6.23~focal-1 \
    wine-staging=6.23~focal-1 \
    wine-staging-i386=6.23~focal-1 \
    wine-staging-amd64=6.23~focal-1

###################################################################################################

if [ -d "/usr/lib/dxc" ]; then
    echo "/usr/lib/dxc" | tee -a /etc/ld.so.conf.d/dxc.conf
    ldconfig
else
    # We need to build dxcompiler from source here
    echo "Building libdxcompiler.so from source."
    DXC_HOME=$HOME/DirectXShaderCompiler
    git clone --depth 1 --branch release-$DXC_VERSION https://github.com/microsoft/DirectXShaderCompiler.git $DXC_HOME
    pushd $DXC_HOME
        git submodule update --init --depth 1
        mkdir build
        pushd build
            cmake .. -GNinja -C ../cmake/caches/PredefinedParams.cmake -DSPIRV_BUILD_TESTS=ON -DCMAKE_BUILD_TYPE=Release
            ninja
            ./bin/dxc -T ps_6_0 ../tools/clang/test/CodeGenSPIRV/passthru-ps.hlsl2spv
            ./bin/dxc -T ps_6_0 -Fo passthru-ps.spv ../tools/clang/test/CodeGenSPIRV/passthru-ps.hlsl2spv -spirv
            ./bin/clang-spirv-tests --spirv-test-root ../tools/clang/test/CodeGenSPIRV/
            #./bin/clang-hlsl-tests --HlslDataDir $PWD/../tools/clang/test/HLSL/
            cp lib/libdxcompiler.so* /usr/lib/
            cp bin/dxc /usr/bin/
            chmod a+x /usr/bin/dxc
            cp -r ../include/dxc /usr/include
        popd
    popd
    rm -rf $DXC_HOME
fi

wget -qO vulkan-sdk.exe https://sdk.lunarg.com/sdk/download/$VULKAN_VERSION.0/windows/VulkanSDK-$VULKAN_VERSION.0-Installer.exe
7z x -y vulkan-sdk.exe -o/xwin/vulkan-sdk
chmod -R a+xr /xwin/vulkan-sdk
rm vulkan-sdk.exe
