# This docker image is mainly for debug/iteration speed purposes 
FROM ubuntu:20.04

LABEL org.opencontainers.image.authors='Legion Labs <devs@legionlabs.com>' \
    org.opencontainers.image.vendor="Legion Labs" \
    org.opencontainers.image.documentation='https://github.com/legion-labs/build-env' \
    org.opencontainers.image.url="https://github.com/legion-labs/build-env" \
    org.opencontainers.image.licenses='Apache-2.0 or MIT' \
    org.opencontainers.image.source="https://github.com/legion-labs/build-env/linux-common"

WORKDIR /install

# Presinstall packages that require interactive user input
RUN apt-get update && DEBIAN_FRONTEND="noninteractive" TZ="America/Toronto"\
    apt-get install -y sudo tzdata keyboard-configuration

# Build layers in order of frequency of changes
COPY ./install/base.sh ./install/helper.sh ./
RUN ./base.sh

COPY ./install/cpp.sh cpp.sh
RUN ./cpp.sh

COPY ./install/libs.sh libs.sh
RUN ./libs.sh

COPY ./install/rust.sh ./install/rust-toolchain.toml ./
RUN ./rust.sh

COPY ./install/libs.sh libs.sh
RUN ./libs.sh

ENV RUSTUP_HOME=/opt/.rustup \
    CARGO_HOME=/opt/.cargo \
    PATH="/opt/.cargo/bin:$PATH" \
    CC_x86_64_pc_windows_msvc="clang-cl" \
    CXX_x86_64_pc_windows_msvc="clang-cl" \
    AR_x86_64_pc_windows_msvc="llvm-lib" \
    LDFLAGS="-fuse-ld=lld" \
    # wine can be quite spammy with log messages and they're generally uninteresting
    WINEDEBUG="-all" \
    # Use wine to run test executables
    CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUNNER="wine" \
    # Note that we only disable unused-command-line-argument here since clang-cl
    # doesn't implement all of the options supported by cl, but the ones it doesn't
    # are _generally_ not interesting.
    CL_FLAGS="-Wno-unused-command-line-argument -fuse-ld=lld-link /imsvc/xwin/crt/include /imsvc/xwin/sdk/include/ucrt /imsvc/xwin/sdk/include/um /imsvc/xwin/sdk/include/shared" \
    # Let cargo know what linker to invoke if you haven't already specified it
    # in a .cargo/config.toml file
    CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER="lld-link" \
    CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS="-Lnative=/xwin/crt/lib/x86_64 -Lnative=/xwin/sdk/lib/um/x86_64 -Lnative=/xwin/sdk/lib/ucrt/x86_64"

ENV CFLAGS_x86_64_pc_windows_msvc="$CL_FLAGS" \
    CXXFLAGS_x86_64_pc_windows_msvc="$CL_FLAGS"

COPY ./install/cargo-installs.sh ./install/tools.toml ./
RUN ./cargo-installs.sh

COPY ./install/cloud.sh ./cloud.sh
RUN ./cloud.sh

# This version is duplicated in the node.sh script, make sure to update both
ENV NODE_VERSION=16.14.0 \
    NVM_DIR=/opt/.nvm

COPY ./install/node.sh ./node.sh
RUN ./node.sh

ENV NODE_PATH="$NVM_DIR/v$NODE_VERSION/lib/node_modules" \
    PATH="$NVM_DIR/versions/node/v$NODE_VERSION/bin:$PATH"

RUN apt-get remove -y --auto-remove && rm -rf /var/lib/apt/lists/*

WORKDIR /

CMD [ "/bin/bash" ]
