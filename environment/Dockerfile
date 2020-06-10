FROM golang:1.14.2-buster as build

COPY ./vendor /vendor
WORKDIR /vendor/ethashproof
RUN ./build.sh

FROM node:14.2.0-buster

WORKDIR /app
COPY --from=build /vendor/ethashproof/cmd/relayer/relayer ./vendor/ethashproof/cmd/relayer/relayer
COPY ./package.json ./package.json
COPY ./yarn.lock ./yarn.lock

RUN yarn

COPY ./lib ./lib
COPY ./index.js ./index.js

ENTRYPOINT ["node", "index.js"]
