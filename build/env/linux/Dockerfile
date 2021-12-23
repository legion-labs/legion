# This docker image is mainly for debug/iteration speed purposes 
FROM ubuntu:20.04

LABEL org.opencontainers.image.authors='Legion Labs <devs@legionlabs.com>' \
    org.opencontainers.image.vendor="Legion Labs" \
    org.opencontainers.image.documentation='https://github.com/legion-labs/build-env' \
    org.opencontainers.image.url="https://github.com/legion-labs/build-env" \
    org.opencontainers.image.licenses='Apache-2.0 or MIT' \
    org.opencontainers.image.source="https://github.com/legion-labs/build-env/linux-common"

# Packer seems to be running commands differently making these not taken into account
RUN apt-get update && DEBIAN_FRONTEND="noninteractive" TZ="America/Toronto"\
    apt-get install -y sudo tzdata keyboard-configuration

COPY ./scripts/build-env.sh build-env.sh

RUN ./build-env.sh

CMD [ "/bin/bash" ]
