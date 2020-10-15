FROM ubuntu:18.04

RUN apt-get update -y  && apt-get install curl gnupg -y

RUN curl -sS https://dl.yarnpkg.com/debian/pubkey.gpg | apt-key add - \
    && echo "deb https://dl.yarnpkg.com/debian/ stable main" | tee /etc/apt/sources.list.d/yarn.list

RUN apt-get update -qq && apt-get install -y -q --no-install-recommends \
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
    python3-pip \
    python3-dev \
    python3-setuptools \
    wget \
    yarn \
    && rm -rf /var/lib/apt/lists/*


ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN curl https://sh.rustup.rs -sSf | \
    sh -s -- -y --no-modify-path --default-toolchain nightly-2020-05-15

RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.35.2/install.sh | bash

ENV GO111MODULE=on 
RUN wget https://golang.org/dl/go1.11.2.linux-amd64.tar.gz
RUN tar -C /usr/local -xzf go1.11.2.linux-amd64.tar.gz
RUN echo 'export PATH=$PATH:/usr/local/go/bin' >> ~/.bashrc

SHELL ["/bin/bash", "--login", "-c", "-i"]
WORKDIR /usr/src

COPY ./requirements.txt . 
RUN pip3 install -r requirements.txt

COPY ./.nvmrc .
RUN nvm install


COPY ./package.json .
RUN node --version
RUN yarn install

COPY . /usr/src/

COPY config* ~/.rainbow

RUN ./index.js prepare

ENV LC_ALL=C.UTF-8
ENV LANG=C.UTF-8
