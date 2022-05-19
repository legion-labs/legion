# Programmer Setup

Legion Engine operates primarily in the gaming tech space, game development have always favored Windows as it is not rare for titles to ship on Windows but also because of the maturity of the game development stack on Windows, but as a cloud centric company we have a strong incentive to use Linux (our servers are going to use Linux for performance, stability, security, and also cost reasons). To be able to have the best of both worlds we chose to focus on Windows 10/11 Pro with Windows Subsystem For Linux 2 (WSL2). We are closely following some interesting development allowing the use of the GPU through WSL that will further favor our development environment but the current investment Microsoft made over the last couple of years on Linux interop puts us in a good spot. Some developers run on Linux as well and our long term goal internally is to have a mixed development environment including MacOs as well.

Our distribution of choice is Ubuntu 20.04 LTS. Ubuntu being a popular distribution it usually enjoys more help resources than other distribution, Canonical also maintains a good relationship with Microsoft which helps improve the WSL experience. For purely server workloads though, Ubuntu is known to be more bloated than some of the more used alternatives, so we might end up down the road switching to a more server centric distribution, but for now let's keep it simple and use Ubuntu across the board.

## Target audience

The programmer setup is targeted toward people making changes to the engine and the pipeline elements, people writing frontend code and rust code. Scripting within the engine doesn't require you to go through the setup process.

## Environment setup

Note that Legion Labs internally uses scoop to install local dependencies and update them, although not a perquisite, it simplifies some of the following steps.

### C/C++ and Rust

### NodeJs

### Docker

## Monorepo tooling

## Cloud access (Legion Labs employee or partner with SSO access)

First you need to install aws cli and a couple of tools.

[Configure you SSO access](https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-sso.html#sso-configure-profile-auto) with the information given to you in the welcome email.

```bash
$ aws configure sso
SSO start URL [None]: [None]: <given to you in the welcome email>
SSO   [None]:ca-central-1
```

The AWS CLI will attempt to open your default browser and begin the login process for your AWS SSO account.

```powershell
scoop install ecr-login
```

Add this to your docker config file `~/.docker/config.json`:

```json
{
  "credHelpers": {
    "616787818023.dkr.ecr.ca-central-1.amazonaws.com": "ecr-login"
  }
}
```

## Build environment docker image

At any given commit you'll be able to run in a virtually similar environment as the build machines run in. To do so you need to locally build the container image the CI uses.

> Legion labs employee or partner: you can also use the same exact image by exporting the MONOREPO_DOCKER_REGISTRY environment variable, to
> `616787818023.dkr.ecr.ca-central-1.amazonaws.com/legion-labs/legion`. You also need the [proper credentials](#Cloud-access) to be access the repository.

To build a container image at the current commit, run the `build/env/setup.sh` or `build/env/setup.bat` scripts on linux or windows respectively. If you are using WSL you can build on the image on windows and use it on WSL and inversely.

```bash

docker image ls

> Identify latest build-env:<sha1 hash>
> REPOSITORY   TAG                                        IMAGE ID       CREATED        SIZE
> build-env    d08a08f410b6d0bcdc9b696468ba88d64e3286ba   0d2c00ba2367   22 hours ago   9.91GB
> build-env    c4bf6f3cf4ca777573bdbde52e1153bba39ecd3b   caab483efbcb   2 days ago     9.91GB

docker run -it --rm build-env:<sha1 hash>

```

You can also use the `build/env/exec.sh` on linux and `build/env/exec.bat` on windows to run a command with your local checkout mapped. The commands uses `target/docker` folder to output cargo artifacts.

For example to build and run tests on linux:

```bash
./build/env/exec.sh cargo m test
```

And to cross compile to windows from linux (using the container image):

```powershell
.\build\env\exec.bat cargo m build --target x86_64-pc-windows-msvc
```
