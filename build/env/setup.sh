#!/bin/bash

SCRIPT_DIR="$(dirname "$0")"
MONOREPO_ROOT="$SCRIPT_DIR/../.."

# These files are outide the context
cp "$MONOREPO_ROOT/rust-toolchain.toml" "$SCRIPT_DIR/install/rust-toolchain.toml"
cp "$MONOREPO_ROOT/.monorepo/tools.toml" "$SCRIPT_DIR/install/tools.toml"

pushd $SCRIPT_DIR 1> /dev/null

CONTAINER_HASH=$(sha1sum install/* | sha1sum | head -c 40)
TAG="build-env:$CONTAINER_HASH"
if [[ $MONOREPO_DOCKER_REGISTRY ]] ; then
    aws ecr get-login-password --region ca-central-1 | docker login --username AWS --password-stdin $MONOREPO_DOCKER_REGISTRY
    ECR_REPO_TAG="$MONOREPO_DOCKER_REGISTRY/$TAG"
    docker pull $ECR_REPO_TAG
    if [[ $? -ne 0 ]] ; then
        docker build . -t $TAG
        docker tag "$TAG" "$ECR_REPO_TAG"
        # we login again here in case our password expired, since the build step takes around 20min
        aws ecr get-login-password --region ca-central-1 | docker login --username AWS --password-stdin $MONOREPO_DOCKER_REGISTRY
        docker push "$ECR_REPO_TAG"
    fi
    echo "ecr_repo_tag=$ECR_REPO_TAG" >> $GITHUB_ENV
else
    if [[ "$(docker images -q $TAG 2> /dev/null)" != "" ]]; then
        echo "Image $TAG already exists"
    else
        docker build . -t $TAG
    fi
fi

rm install/rust-toolchain.toml install/tools.toml

popd 1> /dev/null
