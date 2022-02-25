#!/bin/bash

SCRIPT_DIR="$(dirname "$0")"
MONOREPO_ROOT="$SCRIPT_DIR/../.."

# These files are outide the context
cp "$MONOREPO_ROOT/rust-toolchain.toml" "$SCRIPT_DIR/install/rust-toolchain.toml"
cp "$MONOREPO_ROOT/.monorepo/tools.toml" "$SCRIPT_DIR/install/tools.toml"

pushd $SCRIPT_DIR 1> /dev/null

CONTAINER_HASH=$(sha1sum install/* | sha1sum | head -c 40)
TAG="build-env:$CONTAINER_HASH"
if [[ -n $MONOREPO_DOCKER_REGISTRY ]] ; then
    TAG="$MONOREPO_DOCKER_REGISTRY/$TAG"
    aws ecr get-login-password --region ca-central-1 | docker login --username AWS --password-stdin
    docker pull $TAG
    if [[ $? -ne 0 ]] ; then
        docker build . -t $TAG
        docker push $TAG
    fi
else
    if [[ "$(docker images -q $TAG 2> /dev/null)" != "" ]]; then
        echo "Image $TAG already exists"
    else
        docker build . -t $TAG
    fi
fi

rm install/rust-toolchain.toml install/tools.toml

popd 1> /dev/null

if [[ -z $CI || $CI -eq false ]] ; then
    echo "::set-output name=container::$TAG"
fi
