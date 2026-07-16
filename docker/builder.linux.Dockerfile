# syntax=docker/dockerfile:1
#
# Two selectable final targets: `ubuntu` (glibc) and `alpine` (musl).
# Build with an explicit --target; the last stage in this file is the
# implicit default if you omit it, so don't rely on that ordering.
#
#   docker buildx build --platform linux/amd64,linux/arm64 --target ubuntu -f builder.Dockerfile .
#   docker buildx build --platform linux/amd64,linux/arm64 --target alpine -f builder.Dockerfile .
#
# Architecture (x86_64 / arm64) needs no changes in this file at all —
# it's handled entirely by --platform; every base image below already
# publishes multi-arch manifests.

ARG UBUNTU_VERSION=26.04
ARG ALPINE_VERSION=3.22
ARG PYTHON_VERSION=3.14
ARG GO_VERSION=1.26
ARG NODE_VERSION=24
ARG RUST_VERSION=1.93.1
ARG DOTNET_VERSION=10.0
ARG JAVA_VERSION=25

# =======================================================================
# glibc toolchain sources
# =======================================================================
FROM python:${PYTHON_VERSION}-slim-bookworm AS python-glibc
FROM golang:${GO_VERSION}-bookworm AS go-glibc
FROM node:${NODE_VERSION}-bookworm-slim AS node-glibc
FROM rust:${RUST_VERSION}-slim-bookworm AS rust-glibc
FROM mcr.microsoft.com/dotnet/sdk:${DOTNET_VERSION} AS dotnet-glibc
FROM eclipse-temurin:${JAVA_VERSION}-jdk AS java-glibc

# =======================================================================
# musl toolchain sources
# =======================================================================
FROM python:${PYTHON_VERSION}-alpine${ALPINE_VERSION} AS python-musl
FROM golang:${GO_VERSION}-alpine${ALPINE_VERSION} AS go-musl
FROM node:${NODE_VERSION}-alpine${ALPINE_VERSION} AS node-musl
FROM rust:${RUST_VERSION}-alpine${ALPINE_VERSION} AS rust-musl
FROM mcr.microsoft.com/dotnet/sdk:${DOTNET_VERSION}-alpine AS dotnet-musl
FROM eclipse-temurin:${JAVA_VERSION}-jdk-alpine AS java-musl

# =======================================================================
# Ubuntu (glibc) target
# =======================================================================
FROM ubuntu:${UBUNTU_VERSION} AS ubuntu

RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    ca-certificates \
    build-essential \
    clang \
    && rm -rf /var/lib/apt/lists/*

COPY --from=python-glibc /usr/local /usr/local
COPY --from=go-glibc /usr/local/go /usr/local/go
COPY --from=node-glibc /usr/local /usr/local
COPY --from=rust-glibc /usr/local/cargo /usr/local/cargo
COPY --from=rust-glibc /usr/local/rustup /usr/local/rustup
COPY --from=dotnet-glibc /usr/share/dotnet /usr/share/dotnet
COPY --from=java-glibc /opt/java/openjdk /opt/java/openjdk

ENV PATH="/usr/local/go/bin:/usr/local/cargo/bin:/usr/share/dotnet:/opt/java/openjdk/bin:${PATH}" \
    RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    DOTNET_ROOT=/usr/share/dotnet \
    DOTNET_NOLOGO=true \
    DOTNET_CLI_TELEMETRY_OPTOUT=1 \
    JAVA_HOME=/opt/java/openjdk

# =======================================================================
# Alpine (musl) target
# =======================================================================
FROM alpine:${ALPINE_VERSION} AS alpine

RUN apk add --no-cache \
    curl \
    ca-certificates \
    bash \
    build-base \
    clang

COPY --from=python-musl /usr/local /usr/local
COPY --from=go-musl /usr/local/go /usr/local/go
COPY --from=node-musl /usr/local /usr/local
COPY --from=rust-musl /usr/local/cargo /usr/local/cargo
COPY --from=rust-musl /usr/local/rustup /usr/local/rustup
COPY --from=dotnet-musl /usr/share/dotnet /usr/share/dotnet
COPY --from=java-musl /opt/java/openjdk /opt/java/openjdk

ENV PATH="/usr/local/go/bin:/usr/local/cargo/bin:/usr/share/dotnet:/opt/java/openjdk/bin:${PATH}" \
    RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    DOTNET_ROOT=/usr/share/dotnet \
    DOTNET_NOLOGO=true \
    DOTNET_CLI_TELEMETRY_OPTOUT=1 \
    JAVA_HOME=/opt/java/openjdk
