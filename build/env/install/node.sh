#!/bin/bash

# -e tells the shell to exit if a command exits with an error (except if the exit value is tested in
#    some other way).
# -u tells the shell to treat expanding an unset parameter an error, which helps to catch e.g. typos
#    in variable names.
# -x  tells the shell to print commands and their arguments as they are executed.
set -eux

###################################################################################################

NVM_VERSION=0.39.1
# This version is duplicated in the Dockerfile, make sure to update both
NODE_VERSION=${NODE_VERSION:-'16.14.0'}
PROTOBUF_VERSION=3.19.4

###################################################################################################

export NVM_DIR=/usr/local/nvm
mkdir -p ${NVM_DIR}

wget -qO- https://raw.githubusercontent.com/nvm-sh/nvm/v${NVM_VERSION}/install.sh | bash && . ${NVM_DIR}/nvm.sh
nvm install ${NODE_VERSION}
nvm use ${NODE_VERSION}

export NODE_PATH=$NVM_DIR/v$NODE_VERSION/lib/node_modules
export PATH=$NVM_DIR/versions/node/v$NODE_VERSION/bin:$PATH

npm install -g yarn pbjs pnpm

wget -O protoc.zip https://github.com/protocolbuffers/protobuf/releases/download/v${PROTOBUF_VERSION}/protoc-${PROTOBUF_VERSION}-linux-x86_64.zip
unzip protoc.zip -d protoc
cp protoc/bin/protoc /usr/bin/
chmod a+x /usr/bin/protoc
rm -rf protoc.zip protoc
