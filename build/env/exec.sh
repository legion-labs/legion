#!/bin/bash

SCRIPT_DIR="$(dirname "$0")"
MONOREPO_ROOT="$SCRIPT_DIR/../.."

# These files are outide the context
cp "$MONOREPO_ROOT/rust-toolchain.toml" "$SCRIPT_DIR/install/rust-toolchain.toml"
cp "$MONOREPO_ROOT/.monorepo/tools.toml" "$SCRIPT_DIR/install/tools.toml"

pushd $SCRIPT_DIR 1> /dev/null

CONTAINER_HASH=$(sha1sum install/* | sha1sum | head -c 40)

rm install/rust-toolchain.toml install/tools.toml 1> /dev/null

popd 1> /dev/null

TAG="build-env:$CONTAINER_HASH"
if [[ -n $MONOREPO_DOCKER_REGISTRY ]] ; then
    TAG="$MONOREPO_DOCKER_REGISTRY/$TAG:$CONTAINER_HASH"
    aws ecr get-login-password --region ca-central-1 | docker login --username AWS --password-stdin
    docker pull $TAG
fi

if [[ "$(docker images -q $TAG 2> /dev/null)" == "" ]]; then
    echo "Missing docker image with tag $TAG"
    echo "Run build.sh to build the image"
    exit 1
fi

if [[ -z $CI || $CI -eq false ]] ; then
    docker run --name build-env \
        --rm \
        -v "/var/run/docker.sock":"/var/run/docker.sock" \
        -v "$(realpath $MONOREPO_ROOT)":/github/workspace \
        --workdir /github/workspace \
        $TAG $@
else
    docker run --name build-env \
        --workdir /github/workspace \
        --rm \
        -e CI=true \
        -e MONOREPO_DOCKER_REGISTRY \
        -v "/var/run/docker.sock":"/var/run/docker.sock" \
        -v "/github/work/_temp/_github_home":"/github/home" \
        -v "/github/work/_temp/_github_workflow":"/github/workflow" \
        -v "/github/work/_temp/_runner_file_commands":"/github/file_commands" \
        -v "/github/work/legion/legion":"/github/workspace" \
        $TAG \
        $@
fi
