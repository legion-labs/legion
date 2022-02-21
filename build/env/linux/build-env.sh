#!/bin/bash

# -e tells the shell to exit if a command exits with an error (except if the exit value is tested in some other way).
# -u tells the shell to treat expanding an unset parameter an error, which helps to catch e.g. typos in variable names.
# -x  tells the shell to print commands and their arguments as they are executed.
set -eux

echo "------------------------------------ Install base utils ---------------------------------------"
sudo apt-get update 

export DEBIAN_FRONTEND=noninteractive
export TZ="America/Toronto"

sudo apt-get install -y git make curl wget zip unzip python3 \
    ca-certificates lsb-release software-properties-common jq

echo "--------------------------------------- Build C++ --------------------------------------------"
sudo wget -O - https://apt.kitware.com/keys/kitware-archive-latest.asc 2>/dev/null | gpg --dearmor - | \
    sudo tee /etc/apt/trusted.gpg.d/kitware.gpg >/dev/null
sudo apt-add-repository 'deb https://apt.kitware.com/ubuntu/ focal main'
sudo apt-get update
sudo apt-get install -y make cmake libunwind-dev
sudo wget https://apt.llvm.org/llvm.sh
sudo chmod +x llvm.sh
sudo ./llvm.sh && export LLVM_VERSION=`cat llvm.sh | grep ^CURRENT_LLVM_STABLE= | cut -f2 -d=`
rm ./llvm.sh -f
sudo apt-get install -y \
    pkg-config \
    build-essential \
    libllvm-${LLVM_VERSION}-ocaml-dev \
    libllvm${LLVM_VERSION} \
    llvm-${LLVM_VERSION} \
    llvm-${LLVM_VERSION}-dev \
    llvm-${LLVM_VERSION}-doc \
    llvm-${LLVM_VERSION}-runtime \
    clang-${LLVM_VERSION} \
    clang-tools-${LLVM_VERSION} \
    clang-${LLVM_VERSION}-doc \
    libclang-common-${LLVM_VERSION}-dev \
    libclang-${LLVM_VERSION}-dev \
    libclang1-${LLVM_VERSION} \
    clang-format-${LLVM_VERSION} \
    clangd-${LLVM_VERSION} \
    libfuzzer-${LLVM_VERSION}-dev \
    lldb-${LLVM_VERSION} \
    lld-${LLVM_VERSION} \
    libc++-${LLVM_VERSION}-dev \
    libc++abi-${LLVM_VERSION}-dev \
    libclc-${LLVM_VERSION}-dev \
    python3-lldb-${LLVM_VERSION} \
    nasm \
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
    musl-tools

# Create a link for clang format
sudo ln -s clang-format-${LLVM_VERSION} /usr/bin/clang-format

NINJA_LATEST=$(curl -s https://api.github.com/repos/ninja-build/ninja/releases/latest | \
    jq -r ".assets[] | select(.name | test(\"linux\")) | .browser_download_url")
sudo wget $NINJA_LATEST
sudo unzip ninja-linux.zip
sudo mv ninja /usr/bin/ninja
sudo rm ninja-linux.zip

export CC=clang-$LLVM_VERSION
export CXX=clang++-$LLVM_VERSION
export LDFLAGS="-fuse-ld=lld"

# Persist the environment variable for the profile.
echo "export CC=$CC
export CXX=$CXX
export LDFLAGS=\"$LDFLAGS\"" | sudo tee -a /etc/profile.d/build-cpp.sh 

echo "------------------------------------- Install Rust -------------------------------------------"
export RUSTUP_HOME=$HOME/.rustup
export CARGO_HOME=$HOME/.cargo
export PATH=$CARGO_HOME/bin:$PATH

rustArch='x86_64-unknown-linux-gnu'; rustupSha256='3dc5ef50861ee18657f9db2eeb7392f9c2a6c95c90ab41e45ab4ca71476b4338'; \
    sudo wget https://static.rust-lang.org/rustup/archive/1.24.3/${rustArch}/rustup-init -O rustup-init.sh
echo "${rustupSha256} *rustup-init.sh" | sha256sum -c -
sudo chmod +x rustup-init.sh
./rustup-init.sh -y --no-modify-path --default-host ${rustArch}
rm rustup-init.sh -f
source $HOME/.cargo/env
sudo chmod -R a+w $RUSTUP_HOME $CARGO_HOME
rustup install 1.58.0
rustup install 1.58.1
rustup component add llvm-tools-preview --toolchain '1.58.0'
rustup component add llvm-tools-preview --toolchain '1.58.1'
rustup target add x86_64-unknown-linux-musl --toolchain '1.58.0'
rustup target add x86_64-unknown-linux-musl --toolchain '1.58.1'

# installing some built from source dependencies
cargo install cargo-deny --version "0.11.1" --locked
cargo install mdbook --version "0.4.15" --locked
cargo install sccache --git="https://github.com/diem/sccache.git" --rev=ef50d87a58260c30767520045e242ccdbdb965af
cargo install grcov --version "0.8.6" --locked
cargo install wasm-bindgen-cli --version "0.2.79"

# We need to clean the registery so we don't drag along fetched sources in the image
rm -rf $CARGO_HOME/registry
rm -rf $CARGO_HOME/git

# Persist the environment variable for the profile.
echo "export RUSTUP_HOME=$RUSTUP_HOME
export CARGO_HOME=$CARGO_HOME
export PATH=$CARGO_HOME/bin:\$PATH" | sudo tee -a /etc/profile.d/build-rust.sh

echo "------------------------------------- Install Node ------------------------------------------"
export NVM_DIR=$HOME/nvm
export NVM_VERSION=0.38.0
export NODE_VERSION=16.10.0

mkdir -p ${NVM_DIR}

curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v${NVM_VERSION}/install.sh | bash && . ${NVM_DIR}/nvm.sh
nvm install ${NODE_VERSION}
nvm use ${NODE_VERSION}

export NODE_PATH=$NVM_DIR/v$NODE_VERSION/lib/node_modules
export PATH=$NVM_DIR/versions/node/v$NODE_VERSION/bin:$PATH

npm install -g yarn pbjs pnpm

export PROTOBUF_VERSION=3.19.1
wget -O protoc.zip https://github.com/protocolbuffers/protobuf/releases/download/v${PROTOBUF_VERSION}/protoc-${PROTOBUF_VERSION}-linux-x86_64.zip
unzip protoc.zip -d protoc
sudo cp protoc/bin/protoc /usr/local/bin/
sudo chmod o+x /usr/local/bin/protoc
rm -rf protoc.zip protoc

# Persist the environment variable for the profile.
echo "export NVM_DIR=$NVM_DIR
export NVM_VERSION=$NVM_VERSION
export NODE_VERSION=$NODE_VERSION
export NODE_PATH=$NODE_PATH
export PATH=$NVM_DIR/versions/node/v$NODE_VERSION/bin:\$PATH" | sudo tee -a /etc/profile.d/build-node.sh


echo "------------------------------------- Install VulkanSdk?dxc ------------------------------------------"
wget -qO - http://packages.lunarg.com/lunarg-signing-key-pub.asc | sudo apt-key add -
sudo wget -qO /etc/apt/sources.list.d/lunarg-vulkan-focal.list http://packages.lunarg.com/vulkan/lunarg-vulkan-focal.list
sudo apt-get update
sudo apt-get install -y vulkan-sdk

if [ -d "/usr/lib/dxc" ]; then
    echo "Using pre-installed libdxcompiler.so"
    echo "/usr/lib/dxc" | sudo tee -a /etc/ld.so.conf.d/dxc.conf
    sudo ldconfig
else
    # We need to build dxcompiler from source here
    echo "Building libdxcompiler.so from source."
    DXC_HOME=$HOME/DirectXShaderCompiler
    git clone --depth 1 --branch release-1.6.2110 https://github.com/microsoft/DirectXShaderCompiler.git $DXC_HOME
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
            sudo cp lib/libdxcompiler.so* /usr/lib/
            sudo cp bin/dxc /usr/bin/
            sudo cp -r ../include/dxc /usr/include
        popd
    popd
    rm -rf $DXC_HOME
fi

echo '------------------------------------Install the AWS CLI and Cloud Tools -------------------------------------------------'
curl -fsSL https://apt.releases.hashicorp.com/gpg | sudo apt-key add -
sudo apt-add-repository "deb [arch=$(dpkg --print-architecture)] https://apt.releases.hashicorp.com $(lsb_release -cs) main"

sudo curl -fsSLo /usr/share/keyrings/kubernetes-archive-keyring.gpg https://packages.cloud.google.com/apt/doc/apt-key.gpg
echo "deb [signed-by=/usr/share/keyrings/kubernetes-archive-keyring.gpg] https://apt.kubernetes.io/ kubernetes-xenial main" | \
    sudo tee /etc/apt/sources.list.d/kubernetes.list

curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo apt-key add -
sudo add-apt-repository "deb [arch=amd64] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable"

sudo apt-get update && sudo apt-get install -y terraform kubectl docker-ce docker-ce-cli containerd.io
curl "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "awscliv2.zip"
unzip awscliv2.zip
sudo ./aws/install
rm -rf ./awscliv2.zip ./aws

sudo usermod -aG docker $USER

# Cleanup apt-get caches
sudo rm -rf /var/lib/apt/lists/*

echo "-----------------------------------------------------------------------------------------------"
echo "--------------------------------------- Perfom Checks -----------------------------------------"
echo "-----------------------------------------------------------------------------------------------"

echo "----------------------------------------Check base --------------------------------------------"
git --version
make --version
curl --version
wget --version
zip --version
python3 --version
jq --version
lsb_release -a

echo "----------------------------------------Check C++ --------------------------------------------"
# this should be a local repo...
git clone https://github.com/jameskbride/cmake-hello-world.git
pushd cmake-hello-world
    mkdir build
    pushd build
        cmake .. -G Ninja
        cmake --build .

        echo "int main() { return 0; }" > main.cpp
        $CXX main.cpp
        ./a.out
    popd
popd
rm -rf cmake-hello-world

echo "----------------------------------------Check RUST --------------------------------------------"
rustup show
cargo new test_rust
pushd test_rust
    cargo build
    cargo run
popd
rm -rf test_rust
