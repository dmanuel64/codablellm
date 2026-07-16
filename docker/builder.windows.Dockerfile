# syntax=docker/dockerfile:1
#
# Windows target. Cannot be built alongside builder.Dockerfile's ubuntu/
# alpine targets - Windows containers need the actual Windows kernel, so
# this only builds on a Windows Docker host with the daemon switched to
# "Windows containers" mode, not on Linux CI.
#
#   docker build -f builder.windows.Dockerfile -t codablellm-builder:windows .
#
# Unlike builder.Dockerfile, this doesn't pull per-language toolchains
# from official self-contained images via COPY --from=: the Windows
# container ecosystem doesn't have the same breadth of "one relocatable
# directory" official images Linux does. Chocolatey is the standard
# package manager for Windows Dockerfiles instead.

ARG WINDOWS_VERSION=ltsc2025
ARG PYTHON_VERSION=3.14.0
ARG GO_VERSION=1.26.5
ARG NODE_VERSION=24.9.0
ARG RUST_VERSION=1.93.1
ARG DOTNET_VERSION=10.0
ARG JAVA_VERSION=25

FROM mcr.microsoft.com/windows/servercore:${WINDOWS_VERSION}
ARG PYTHON_VERSION
ARG GO_VERSION
ARG NODE_VERSION
ARG RUST_VERSION
ARG DOTNET_VERSION
ARG JAVA_VERSION

SHELL ["powershell", "-NoLogo", "-Command", "$ErrorActionPreference = 'Stop'; $ProgressPreference = 'SilentlyContinue';"]

RUN Set-ExecutionPolicy Bypass -Scope Process -Force; \
    [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; \
    Invoke-Expression ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))

RUN choco install -y --no-progress python3 --version=$env:PYTHON_VERSION; \
    choco install -y --no-progress golang --version=$env:GO_VERSION; \
    choco install -y --no-progress nodejs --version=$env:NODE_VERSION; \
    choco install -y --no-progress rust --version=$env:RUST_VERSION; \
    choco install -y --no-progress dotnet-sdk --version=$env:DOTNET_VERSION; \
    choco install -y --no-progress temurin --version=$env:JAVA_VERSION; \
    choco install -y --no-progress visualstudio2022buildtools; \
    choco install -y --no-progress visualstudio2022-workload-vctools; \
    choco install -y --no-progress llvm

ENTRYPOINT ["powershell"]
