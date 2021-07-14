export AWS_PROFILE=github_gov
aws sts assume-role --role-arn arn:aws:iam::550877636976:role/AutomationRole --role-session-name test --region eu-central-1 > tmp.json
export AWS_ACCESS_KEY_ID=$(cat tmp.json | jq .Credentials.AccessKeyId);
export AWS_SECRET_ACCESS_KEY=$(cat tmp.json | jq .Credentials.SecretAccessKey);
export AWS_SESSION_TOKEN=$(cat tmp.json | jq .Credentials.SessionToken);
unset AWS_ACCESS_KEY_ID AWS_SECRET_ACCESS_KEY AWS_SESSION_TOKEN
rm tmp.json
aws ecr get-login-password --region ca-central-1 | docker login --username AWS --password-stdin 616787818023.dkr.ecr.ca-central-1.amazonaws.com