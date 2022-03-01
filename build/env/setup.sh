#!/bin/bash

SCRIPT_DIR="$(dirname "$0")"
MONOREPO_ROOT="$SCRIPT_DIR/../.."

# These files are outide the context
cp "$MONOREPO_ROOT/rust-toolchain.toml" "$SCRIPT_DIR/install/rust-toolchain.toml"
cp "$MONOREPO_ROOT/.monorepo/tools.toml" "$SCRIPT_DIR/install/tools.toml"

pushd $SCRIPT_DIR 1> /dev/null

if [[ -z $IMAGE_NAME ]]; then
    IMAGE_NAME="build-env"
fi
if [[ -z $IMAGE_TAG ]]; then
    IMAGE_TAG=$(sha1sum Dockerfile install/* | sha1sum | head -c 40)
fi
exit_code=0
if [[ $MONOREPO_DOCKER_REGISTRY ]] ; then
    IMAGE="$MONOREPO_DOCKER_REGISTRY/$IMAGE_NAME:$IMAGE_TAG"
    echo "Using image $IMAGE"
    aws ecr get-login-password --region ca-central-1 | docker login --username AWS --password-stdin $MONOREPO_DOCKER_REGISTRY
    PUSH=0
    if [[ $IMAGE_TAG == "latest" ]]; then
        echo "Building image $IMAGE"
        set -e
        docker build . -t $IMAGE
        exit_code=$?
        PUSH=1
    else
        docker manifest inspect $IMAGE &> /dev/null
        if [[ $? -ne 0 ]]; then
            echo "Building image $IMAGE"
            # Pull latest image in case we can share some layers
            docker build . -t $IMAGE
            exit_code=$?
            PUSH=1
        fi
    fi
    if [[ $PUSH -eq 1 ]]; then
        # we login again here in case our password expired, since the build step takes around 20min
        aws ecr get-login-password --region ca-central-1 | docker login --username AWS --password-stdin $MONOREPO_DOCKER_REGISTRY
        docker push "$IMAGE"
        exit_code=$?
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

exit $exit_code
