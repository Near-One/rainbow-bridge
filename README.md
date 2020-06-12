<div align="center">

  <h1><code>Rainbow Bridge</code></h1>

  <p>
    <strong>Ethereum to Near trustless, fully decentralized, bidirectional bridge</strong>
  </p>

  <p>
    <a href="https://travis-ci.com/near/rainbow-bridge"><img src="https://travis-ci.com/near/rainbow-bridge.svg?branch=master" alt="Travis Build" /></a>
  </p>
  <h3>
        <a href="https://github.com/near/rainbow-bridge#installation">Installation</a>
        <span> | </span>
        <a href="https://github.com/near/rainbow-bridge#usage">Usage</a>
        <span> | </span>
        <a href="https://github.com/near/rainbow-bridge/tree/master/docs#documentation">Documentation</a>
        <span> | </span>
        <a href="https://github.com/near/rainbow-bridge#examples">Examples</a>
  </h3>
</div>

## Installation

TODO: Need to publish it to npm.

## Usage
To learn the commands that you can use with the rainbow bridge run
```bash
node index.js --help
```

Parameters of each command can be specified through environment variables, command line arguments, entries in the `~/.rainbowup/config.json` config file, or the default value will be used -- in that priority.
If argument is not provided and there is no default value the program will not execute.

If script successfully executes a command then each parameter provided through the command line argument will be
written into the config file. Additionally, if scripts generates new parameters (e.g. it deploys a contract to Ethereum
and obtains its address) will also be written into the config file. Arguments should not be specified multiple times. 

Note, you can use environment variables to pass sensitive data which will not lead to it being written into the config file.


## Usage example
Go to `environment` folder.

### Launching blockchains locally

First start the services that will emulate locally the NEAR and the Ethereum blockchains:
=======
## Configs and flags

There are three layers of configuration, overriding each other with highest priorities first:

1. Environment variables
2. Flags passed throught the command line
3. Configuration files in ~/.rainbowup/config.json

More about specific configurations:

**(TODO)**



## Local test run
To locally test the bridge run:
```bash
node index.js clean
node index.js prepare
node index.js start near-node
node index.js start ganache
```
In a separate window (this is a temporary workaround for [#96](https://github.com/near/rainbow-bridge/issues/96)):
```bash
./scripts/start_ganache.sh
```

### Initializing the contracts

First let's initialize the contracts that bridge needs to function:
```bash
node index.js init-near-contracts
node index.js init-eth-ed25519
node index.js init-near2eth-client
```

Now, let's set up token on Ethereum blockchain that we can transfer to NEAR blockchain (this can be your own token).
```bash
node index.js init-eth-erc20
node index.js init-eth-locker
```
Now, let's initialize token on NEAR blockchain that will mirror the token on Ethereum side.
```bash
node index.js init-near-fun-token
```

### Starting the services
Now start the services that will relay the information between the chains:
```bash
node index.js start eth-relay
node index.js start near-relay
```

Note, you can observe the logs of the relays by running:
```bash
pm2 logs
```
<<<<<<< HEAD

### Transferring tokens 
Finally, let's transfer some tokens 
```bash
node index.js transfer-eth-erc20-to-near --amount 1 --eth-sender-sk 0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501200 --near-receiver-account alice.test.near --near-master-account nearfuntoken
```
Note, when we deployed ERC20 to the Ethereum blockchain we have minted a large number of tokens to the default master
key of Ganache, so we have transferred ERC20 tokens from it to `alice.test.near`.
Notice that we are using `nearfuntoken` account here to pay for the NEAR gas fees, any account for which we know a secret key would've worked too.
=======
You must observe blocks being submitted.

Docker:

## Currently we have the following docker options:

1. Rainbow Docker image containing rainbowup ready for running
	- run the rainbowup docker image with a custom command
2. A development docker compose setup (docker-compose-dev.yml)
	- ganache
	- local near node
	- eth-relay
3. A production docker compose setup (docker-compose-prod.yml)
	- eth-relay

## Running the docker setup:

1. One options is to adapt the current config.json specified in the root folder of the project and build a new image.
2. Specifying the configuration flags through environment variables.

We recommend a usage of both, encouraging using the config.json for common configurations, while passing the secrets through environment variables.

Examples:

```
# Creating a docker image
docker build .

# Running the development env with config setup
docker-compose -f docker-compose-dev.yml up

# Running the development env with ENV overrides 
docker-compose -f docker-compose-dev.yml up -e MASTER_SK=<key> -e ...

# Running the production env just use:
docker-compose -f docker-compose-prod.yml instead
```

