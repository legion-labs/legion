# escape=`

FROM mcr.microsoft.com/windows:20H2

LABEL org.opencontainers.image.authors='Legion Labs <devs@legionlabs.com>' `
    org.opencontainers.image.vendor="Legion Labs" `
    org.opencontainers.image.documentation='https://github.com/legion-labs/build-env' `
    org.opencontainers.image.url="https://github.com/legion-labs/build-env" `
    org.opencontainers.image.licenses='Apache-2.0 or MIT' `
    org.opencontainers.image.source="https://github.com/legion-labs/build-env/windows-common"

SHELL ["powershell", "-NoLogo", "-NonInteractive", "-Command", "$ErrorActionPreference = 'Stop'; $ProgressPreference = 'Continue'; $verbosePreference='Continue';"]

COPY ./scripts/build-env.ps1 build-env.ps1

RUN Set-ExecutionPolicy Bypass -scope CurrentUser; `
    ./build-env.ps1

ENTRYPOINT [ "powershell", "-NoLogo", "-Command" ]
CMD [ "powershell", "-NoExit" ]
