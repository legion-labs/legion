#!/bin/bash

# -e tells the shell to exit if a command exits with an error (except if the exit value is tested in
#    some other way).
# -u tells the shell to treat expanding an unset parameter an error, which helps to catch e.g. typos
#    in variable names.
# -x  tells the shell to print commands and their arguments as they are executed.
set -eux

###################################################################################################

TERRAFORM_VERSION=1.1.6
KUBECTL_VERSION=1.23.4-00
DOCKER_VERSION=5:20.10.12~3-0~ubuntu-focal
CONTAINERD_VERSION=1.4.12-1
AWS_CLI_VERSION=2.4.20

###################################################################################################

source "$(dirname "$0")/helper.sh"

###################################################################################################

curl -fsSL https://apt.releases.hashicorp.com/gpg | apt-key add -
apt-add-repository "deb [arch=$(dpkg --print-architecture)] https://apt.releases.hashicorp.com $DISTRO_CODENAME main"

curl -fsSLo /usr/share/keyrings/kubernetes-archive-keyring.gpg https://packages.cloud.google.com/apt/doc/apt-key.gpg
echo "deb [signed-by=/usr/share/keyrings/kubernetes-archive-keyring.gpg] https://apt.kubernetes.io/ kubernetes-xenial main" | \
    tee /etc/apt/sources.list.d/kubernetes.list

curl -fsSL https://download.docker.com/linux/ubuntu/gpg | apt-key add -
add-apt-repository "deb [arch=amd64] https://download.docker.com/linux/ubuntu $DISTRO_CODENAME stable"

apt-get update && apt-get install -y --no-install-recommends \
    terraform=$TERRAFORM_VERSION \
    kubectl=$KUBECTL_VERSION \
    docker-ce-cli=$DOCKER_VERSION \
    containerd.io=$CONTAINERD_VERSION

###################################################################################################

curl "https://awscli.amazonaws.com/awscli-exe-linux-x86_64-$AWS_CLI_VERSION.zip" -o "/tmp/awscliv2.zip"
unzip /tmp/awscliv2.zip -d /tmp
set +e
/tmp/aws/install
if [ $? -ne 0 ]; then
    set -e
    /tmp/aws/install --update
else
    set -e
fi
rm -rf /tmp/awscliv2.zip /tmp/aws
