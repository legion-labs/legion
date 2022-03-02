#!/bin/bash

# -e tells the shell to exit if a command exits with an error (except if the exit value is tested in
#    some other way).
# -u tells the shell to treat expanding an unset parameter an error, which helps to catch e.g. typos
#    in variable names.
# -x  tells the shell to print commands and their arguments as they are executed.
set -eux

###################################################################################################

DASEL_VERSION=1.22.1

###################################################################################################

apt-get update && apt-get install -y --no-install-recommends \
    lsb-release pciutils \
    curl wget \
    tar zip unzip p7zip-full \
    git \
    python3 \
    gpg-agent ca-certificates software-properties-common \
    jq

# Dazel is used to parse anything other than Json (Especially toml used by Rust).
wget -qO /usr/bin/dasel https://github.com/TomWright/dasel/releases/download/v$DASEL_VERSION/dasel_linux_amd64
chmod a+x /usr/bin/dasel
