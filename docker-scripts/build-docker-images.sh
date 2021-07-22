#!/bin/bash
echo " === Building docker images Start ==="
IMAGE_PREFIX="legionlabs"
for i in $(find ./ -name 'Dockerfile')
do
    DOCKERFILE_FULL_DIRECTORY_NAME=$(dirname ${i})

    APP_FULL_PATH_NAME=$(dirname ${i})
    APP_DIRECTORY_NAME="$(basename $APP_FULL_PATH_NAME)"
    PACKAGE_NAME=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | select(.manifest_path|test("'$APP_DIRECTORY_NAME'"))| .name')
    PACKAGE_VERSION=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | select(.manifest_path|test("'$APP_DIRECTORY_NAME'"))| .version')
   
    REPOSITORY_NAME="legionlabs/$PACKAGE_NAME"
    
    docker build -t  "$REPOSITORY_NAME:$PACKAGE_VERSION" -f $i .
done
echo " === Building docker images Done ==="
