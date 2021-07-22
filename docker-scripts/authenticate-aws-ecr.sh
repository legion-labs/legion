#!/bin/bash
AWS_REGION=$1
ACCOUNT_ID=$(aws ecr describe-registry | jq -r .registryId)
AWS_ECR_REGISTRY_URL="${ACCOUNT_ID}.dkr.ecr.${AWS_REGION}.amazonaws.com"

echo " === Login to ECR ==="
aws ecr get-login-password --region $AWS_REGION | docker login --username AWS --password-stdin $AWS_ECR_REGISTRY_URL
echo " === Login Done ==="