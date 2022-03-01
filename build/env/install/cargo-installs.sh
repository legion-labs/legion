#!/bin/bash

# -e tells the shell to exit if a command exits with an error (except if the exit value is tested in
#    some other way).
# -u tells the shell to treat expanding an unset parameter an error, which helps to catch e.g. typos
#    in variable names.
# -x  tells the shell to print commands and their arguments as they are executed.
set -eux

###################################################################################################

CARGO_FETCHER_VERSION=0.12.1

###################################################################################################

INSTALL_LIST=($(dasel select -f tools.toml -m 'cargo-installs.-'))

for tool in "${INSTALL_LIST[@]}"
do
    echo "Installing $tool"
    TOOL_VERSION=$(dasel select -f tools.toml "cargo-installs.$tool.version")
    TOOL_IN_GIT=$(dasel select -f tools.toml  "cargo-installs.$tool" | grep -c 'git =' || true)
    if [[ $TOOL_IN_GIT -eq 0 ]] ; then
        cargo install $tool --version $TOOL_VERSION --locked
    else
        TOOL_GIT_URL=$(dasel select -f tools.toml "cargo-installs.$tool.git")
        TOOL_GIT_REV=$(dasel select -f tools.toml "cargo-installs.$tool.rev")
        cargo install $tool --git $TOOL_GIT_URL --rev $TOOL_GIT_REV --locked
    fi
done

###################################################################################################

CARGO_FETCHER_PREFIX="cargo-fetcher-$CARGO_FETCHER_VERSION-x86_64-unknown-linux-musl"
wget -qO- https://github.com/EmbarkStudios/cargo-fetcher/releases/download/$CARGO_FETCHER_VERSION/$CARGO_FETCHER_PREFIX.tar.gz |
    tar -xzv -C $CARGO_HOME/bin --strip-components=1 $CARGO_FETCHER_PREFIX/cargo-fetcher


###################################################################################################