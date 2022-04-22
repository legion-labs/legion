#!/bin/bash

# -e tells the shell to exit if a command exits with an error (except if the exit value is tested in
#    some other way).
# -u tells the shell to treat expanding an unset parameter an error, which helps to catch e.g. typos
#    in variable names.
# -x  tells the shell to print commands and their arguments as they are executed.
set -eux

###################################################################################################

VULKAN_VERSION=1.3.204
VULKAN_PATCH_VERSION=1
DXC_VERSION=1.6.2112

###################################################################################################

source "$(dirname "$0")/helper.sh"

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
    libwayland-dev \
    libxkbcommon-dev \
    fuse3 \
    libfuse3-dev \
    libudev-dev \
    vulkan-sdk

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

# check here for the available windows versions https://vulkan.lunarg.com/sdk/home
# Note that there is a minor version appended, and LunarG deletes sometimes the older minor version...
# If you ever get a CI failure at this line check the website and update accordingly.
# We periodically build images from scratch to detect stale dependencies.
wget -qO vulkan-sdk.exe https://sdk.lunarg.com/sdk/download/$VULKAN_VERSION.$VULKAN_PATCH_VERSION/windows/VulkanSDK-$VULKAN_VERSION.$VULKAN_PATCH_VERSION-Installer.exe
7z x -y vulkan-sdk.exe -o/xwin/vulkan-sdk
chmod -R a+xr /xwin/vulkan-sdk
rm vulkan-sdk.exe
