#!/bin/bash

SCRIPT_DIR="$(dirname "$0")"
MONOREPO_ROOT="$SCRIPT_DIR/../.."

# These files are outide the context
cp "$MONOREPO_ROOT/rust-toolchain.toml" "$SCRIPT_DIR/install/rust-toolchain.toml"
cp "$MONOREPO_ROOT/.monorepo/tools.toml" "$SCRIPT_DIR/install/tools.toml"

pushd $SCRIPT_DIR 1> /dev/null

IMAGE_NAME="build-env"
IMAGE_TAG=$(sha1sum install/* | sha1sum | head -c 40)
if [[ $MONOREPO_DOCKER_REGISTRY ]] ; then
    IMAGE="$MONOREPO_DOCKER_REGISTRY/$IMAGE_NAME:$IMAGE_TAG"
    aws ecr get-login-password --region ca-central-1 | docker login --username AWS --password-stdin $MONOREPO_DOCKER_REGISTRY
    docker manifest inspect $IMAGE &> /dev/null
    if [[ $? -ne 0 ]] ; then
        docker build . -t $IMAGE
        # we login again here in case our password expired, since the build step takes around 20min
        aws ecr get-login-password --region ca-central-1 | docker login --username AWS --password-stdin $MONOREPO_DOCKER_REGISTRY
        docker push "$IMAGE"
    fi
    echo "image=$IMAGE" >> $GITHUB_ENV
else
    IMAGE="$IMAGE_NAME:$IMAGE_TAG"
    if [[ "$(docker images -q $IMAGE 2> /dev/null)" != "" ]]; then
        echo "Image $IMAGE already exists"
    else
        docker build . -t $IMAGE
    fi
fi

rm install/rust-toolchain.toml install/tools.toml

popd 1> /dev/null
