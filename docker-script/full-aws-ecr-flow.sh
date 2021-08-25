#!/bin/bash
AWS_REGION=$1
sh docker-script/build-docker-images.sh
sh docker-script/authenticate-aws-ecr.sh $AWS_REGION
sh docker-script/create-aws-ecr-repository.sh $AWS_REGION
sh docker-script/tag-push-docker-images.sh $AWS_REGION
