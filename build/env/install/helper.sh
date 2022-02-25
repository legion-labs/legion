#!/bin/bash

DISTRO=$(lsb_release -is)
DISTRO_VERSION=$(lsb_release -sr)
DISTRO_CODENAME=$(lsb_release -cs)
DISTRO_NAME_VERSION="${DISTRO}_${DISTRO_VERSION}"

