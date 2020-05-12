FROM node:14.2

COPY ./package.json ./package.json
COPY ./yarn.lock ./yarn.lock

RUN yarn

COPY . .

ENTRYPOINT ["node", "index.js"]
