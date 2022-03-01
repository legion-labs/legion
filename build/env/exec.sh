#!/bin/bash

SCRIPT_DIR="$(dirname "$0")"
MONOREPO_ROOT="$SCRIPT_DIR/../.."

# These files are outide the context
cp "$MONOREPO_ROOT/rust-toolchain.toml" "$SCRIPT_DIR/install/rust-toolchain.toml"
cp "$MONOREPO_ROOT/.monorepo/tools.toml" "$SCRIPT_DIR/install/tools.toml"

pushd $SCRIPT_DIR 1> /dev/null

IMAGE_TAG=$(sha1sum Dockerfile install/* | sha1sum | head -c 40)

rm install/rust-toolchain.toml install/tools.toml 1> /dev/null

popd 1> /dev/null

IMAGE_NAME="build-env"
IMAGE="$IMAGE_NAME:$IMAGE_TAG"
if [[ -n $MONOREPO_DOCKER_REGISTRY ]] ; then
    IMAGE="$MONOREPO_DOCKER_REGISTRY/$IMAGE_NAME:$IMAGE_TAG"
    aws ecr get-login-password --region ca-central-1 | docker login --username AWS --password-stdin
    docker pull $IMAGE
fi

if [[ "$(docker images -q $IMAGE 2> /dev/null)" == "" ]]; then
    echo "Missing docker image with tag $IMAGE"
    echo "Run build.sh to build the image"
    exit 1
fi

docker run --name build-env \
    -it --rm \
    -e CARGO_TARGET_DIR="target\docker" \
    -v "/var/run/docker.sock":"/var/run/docker.sock" \
    -v "$(realpath $MONOREPO_ROOT)":/github/workspace \
    --workdir /github/workspace \
    $IMAGE