#!/bin/bash

AWS_REGION=$1
echo " === Create repositories in AWS ECR ==="

for i in $(find ./ -name 'Dockerfile')
do
    APP_FULL_PATH_NAME=$(dirname ${i})
    APP_DIRECTORY_NAME="$(basename $APP_FULL_PATH_NAME)"

    # extract package name from the manifest file
    PACKAGE_NAME=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | select(.manifest_path|test("'$APP_DIRECTORY_NAME'"))| .name')
    REPOSITORY_NAME="legionlabs/$PACKAGE_NAME"
    
    EXISTING_REPOSITORY=$(aws ecr describe-repositories | jq -r '.repositories[] | select(.repositoryName == "'$REPOSITORY_NAME'") | .repositoryName')
    if [ "$REPOSITORY_NAME" != "$EXISTING_REPOSITORY" ]; then
        echo "Create repository $REPOSITORY_NAME"
        result=$(aws ecr create-repository --repository-name $REPOSITORY_NAME --image-scanning-configuration scanOnPush=true --region $AWS_REGION)
    fi
done
echo " === Create repositories in AWS ECR Done ==="
