ARG BASE_IMAGE_VERSION=26.04
ARG PYTHON_VERSION=3.14
ARG GO_VERSION=1.26
ARG NVM_VERSION=0.40.4
ARG NODE_VERSION=24
ARG RUST_VERSION=1.93.1

FROM ubuntu:${BASE_IMAGE_VERSION}

RUN apt-get update && apt-get install \
    curl \
    python=${PYTHON_VERSION} \
    golang-go=${GO_VERSION} \
    && rm -rf /var/lib/apt/lists/*

# Download and install nvm:
RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v${NVM_VERSION}/install.sh | bash
# in lieu of restarting the shell
RUN \. "$HOME/.nvm/nvm.sh"
# Download and install Node.js:
RUN nvm install ${NODE_VERSION}

# Install Rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --no-modify-path --default-toolchain ${RUST_VERSION}

ENTRYPOINT [ "/bin/bash" ]