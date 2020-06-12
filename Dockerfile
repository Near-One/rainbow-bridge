FROM ubuntu:18.04

<<<<<<< HEAD
COPY . /usr/src/

RUN apt-get update && apt-get upgrade -y
RUN apt-get update -qq && apt-get install -y \
=======
RUN apt-get update && apt-get install -y \
>>>>>>> Dockerization of rainbowup
    build-essential \
    curl \
    clang \
    cmake \
    git \
    g++ \
    nodejs \
    libssl-dev \
    llvm \
<<<<<<< HEAD
=======
    netcat \
>>>>>>> Dockerization of rainbowup
    pkg-config \
    python3 \
    && rm -rf /var/lib/apt/lists/*


ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN curl https://sh.rustup.rs -sSf | \
    sh -s -- -y --no-modify-path --default-toolchain nightly-2020-05-15

<<<<<<< HEAD

=======
COPY . /usr/src/
>>>>>>> Dockerization of rainbowup
COPY config* ~/.rainbowup

WORKDIR /usr/src/environment
RUN node index.js prepare
