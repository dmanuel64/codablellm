# syntax=docker/dockerfile:1
#
# Adapted from Ghidra's official docker/Dockerfile: https://github.com/NationalSecurityAgency/ghidra/blob/master/docker/Dockerfile

ARG ALPINE_VERSION=3.20
ARG GHIDRA_VERSION=12.1.2
ARG GHIDRA_RELEASE_DATE=20260605
ARG GHIDRA_SHA256=b62e81a0390618466c019c60d8c2f796ced2509c4c1aea4a37644a77272cf99d

FROM alpine:${ALPINE_VERSION} AS fetch
ARG GHIDRA_VERSION
ARG GHIDRA_RELEASE_DATE
ARG GHIDRA_SHA256

RUN apk add --no-cache curl unzip

WORKDIR /download
RUN curl -fsSL -o ghidra.zip \
    "https://github.com/NationalSecurityAgency/ghidra/releases/download/Ghidra_${GHIDRA_VERSION}_build/ghidra_${GHIDRA_VERSION}_PUBLIC_${GHIDRA_RELEASE_DATE}.zip" \
    && echo "${GHIDRA_SHA256}  ghidra.zip" | sha256sum -c - \
    && unzip -q ghidra.zip \
    && rm ghidra.zip \
    && mv "ghidra_${GHIDRA_VERSION}_PUBLIC" /ghidra-release

# =======================================================================
# Ghidra Dockerfile
# =======================================================================
FROM alpine:${ALPINE_VERSION} AS base

LABEL org.opencontainers.image.title="ghidra" \
    org.opencontainers.image.description="Docker image for Ghidra" \
    org.opencontainers.image.source="https://github.com/NationalSecurityAgency/ghidra" \
    org.opencontainers.image.licenses="Apache 2.0"

# Configure user, entrypoint, and some env vars first, before making the image larger with dependencies
# so that we can keep the image size as small as possible.
RUN addgroup -g 1001 -S ghidra && adduser -u 1001 -S ghidra -G ghidra
ENTRYPOINT ["/bin/bash", "/ghidra/docker/entrypoint.sh"]
# Set JAVA_HOME so that we don't need to do this manually when Ghidra is first started.
ENV JAVA_HOME=/usr/lib/jvm/java-21-openjdk
ENV LD_LIBRARY_PATH=/usr/lib/jvm/java-21-openjdk/lib/:/usr/lib/jvm/java-21-openjdk/lib/server/
WORKDIR /ghidra

# update and install dependencies used to both build and run ghidra
RUN apk update \
    && apk add openjdk21 python3 py3-pip \
    bash gcompat \
    fontconfig msttcorefonts-installer \
    linux-headers libressl-dev \
    && update-ms-fonts \
    && pip install --break-system-packages --upgrade pip

FROM base AS build

# install additional dependencies used to build ghidra
RUN apk add gradle \
    python3-dev \
    alpine-sdk \
    build-base \
    gcc g++ make libc-dev zlib-dev musl-dev \
    zip readline-dev

# copy the fetched release in, in place of the official Dockerfile's
# `COPY . .` from a pre-extracted release directory.
COPY --from=fetch /ghidra-release/ .

# build postgres and install pyghidra
RUN /ghidra/Ghidra/Features/BSim/support/make-postgres.sh \
    && python3 -m venv /ghidra/venv \
    && /ghidra/venv/bin/python3 -m pip install --no-index -f /ghidra/Ghidra/Features/PyGhidra/pypkg/dist pyghidra \
    && mkdir /ghidra/repositories && mkdir /ghidra/bsim_datadir


FROM base AS runtime

# install additional dependencies needed for running ghidra
RUN apk add openssl openssh-client \
    xhost musl-locales musl-locales-lang

USER ghidra
WORKDIR /ghidra
COPY --chown=ghidra:ghidra --from=build /ghidra /ghidra
