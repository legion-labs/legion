#!/bin/bash
AWS_REGION=$1
sh docker-scripts/build-docker-images.sh
sh docker-scripts/authenticate-aws-ecr.sh $AWS_REGION
sh docker-scripts/create-aws-ecr-repository.sh $AWS_REGION
sh docker-scripts/tag-push-docker-images.sh $AWS_REGION
