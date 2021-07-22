#!/bin/bash
AWS_REGION=$1
ACCOUNT_ID=$(aws ecr describe-registry | jq -r .registryId)
AWS_ECR_REGISTRY_URL="${ACCOUNT_ID}.dkr.ecr.${AWS_REGION}.amazonaws.com"
IMAGE_PREFIX="legionlabs"
echo " === Tag and push to AWS ECR Start ==="

# Search for images from legionlabs only
for i in $(find ./ -name 'Dockerfile')
do
    APP_FULL_PATH_NAME=$(dirname ${i})
    APP_DIRECTORY_NAME="$(basename $APP_FULL_PATH_NAME)"
    # extract package name from the manifest file
    PACKAGE_NAME=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | select(.manifest_path|test("'$APP_DIRECTORY_NAME'"))| .name')
    PACKAGE_VERSION=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | select(.manifest_path|test("'$APP_DIRECTORY_NAME'"))| .version')
   
    REPOSITORY_NAME="legionlabs/$PACKAGE_NAME"

    # check image from the local registry
    LOCAL_IMAGE=$(docker images "$REPOSITORY_NAME:$PACKAGE_VERSION" --format {{.Repository}}:{{.Tag}})
    if [ -n "$LOCAL_IMAGE" ];
    then
        # check remote image tag
        REMOTE_IMAGE_TAG=$(aws ecr list-images --repository-name=$REPOSITORY_NAME --filter="tagStatus=TAGGED" | jq -r '.imageIds[-1].imageTag')
        ABC=$(aws ecr list-images --repository-name legionlabs/simplewebserver --filter="tagStatus=TAGGED" | jq -r '.imageIds[] | select(.imageTag|test("'$PACKAGE_VERSION'"))|.imageTag')
        echo "VERSION [$ABC]"
        if [ "$LOCAL_IMAGE" != "$REPOSITORY_NAME:$REMOTE_IMAGE_TAG" ]
        then
            echo "Tag and push image and tags"
            docker tag $LOCAL_IMAGE $AWS_ECR_REGISTRY_URL/$LOCAL_IMAGE
            docker push $AWS_ECR_REGISTRY_URL/$LOCAL_IMAGE
        else
            echo "$LOCAL_IMAGE already exist"
        fi
    fi
done
echo " === Tag and push to AWS ECR Done ==="
