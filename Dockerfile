FROM ubuntu:18.04

RUN apt-get update && apt-get install -y \
    COPY . /usr/src/

RUN apt-get update -qq && apt-get install -y \
    build-essential \
    curl \
    clang \
    cmake \
    git \
    g++ \
    libssl-dev \
    llvm \
    netcat \
    pkg-config \
    python3 \
    && rm -rf /var/lib/apt/lists/*

RUN curl -sL https://deb.nodesource.com/setup_12.x | bash -
RUN apt-get install nodejs && npm -g install ganache-cli

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN curl https://sh.rustup.rs -sSf | \
    sh -s -- -y --no-modify-path --default-toolchain nightly-2020-05-15

COPY . /usr/src/
COPY config* ~/.rainbow

WORKDIR /usr/src
RUN node index.js prepare
