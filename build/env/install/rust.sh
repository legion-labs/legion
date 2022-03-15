#!/bin/bash

# -e tells the shell to exit if a command exits with an error (except if the exit value is tested in
#    some other way).
# -u tells the shell to treat expanding an unset parameter an error, which helps to catch e.g. typos
#    in variable names.
# -x  tells the shell to print commands and their arguments as they are executed.
set -eux

###################################################################################################

RUSTUP_VERSION=1.24.3
RUSTUP_HASH="3dc5ef50861ee18657f9db2eeb7392f9c2a6c95c90ab41e45ab4ca71476b4338"
RUST_VERSION=$(dasel select -f rust-toolchain.toml 'toolchain.channel')

###################################################################################################

wget -O rustup-init.sh https://static.rust-lang.org/rustup/archive/$RUSTUP_VERSION/x86_64-unknown-linux-gnu/rustup-init
echo "$RUSTUP_HASH *rustup-init.sh" | sha256sum -c -
chmod +x rustup-init.sh
./rustup-init.sh -y --no-modify-path --default-toolchain $RUST_VERSION
rm rustup-init.sh -f
source $CARGO_HOME/env
chmod -R a+w $RUSTUP_HOME $CARGO_HOME
rustup component add llvm-tools-preview
rustup target add x86_64-unknown-linux-musl x86_64-pc-windows-msvc
