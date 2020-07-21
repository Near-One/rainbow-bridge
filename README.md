<div align="center">

  <h1><code>Rainbow Bridge</code></h1>

  <p>
    <strong>Ethereum to Near trustless, fully decentralized, bidirectional bridge</strong>
  </p>

  <p>
    <a href="https://buildkite.com/nearprotocol/rainbow-bridge"><img src="https://badge.buildkite.com/a3dcd9711ef855a7ea6dc80453828ad73d7a669b9a925889ad.svg?branch=master" alt="Buildkite Build" /></a>
    <a href="https://npmjs.com/rainbow-bridge-cli"><img alt="npm" src="https://img.shields.io/npm/v/rainbow-bridge-cli.svg?style=flat-square"></a>
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

## Pre-requisites
The current version of CLI is all-in-one package -- it is used both for production and testing. As a result, even if you
need CLI only for the token transfer you need to install all testing dependencies. This will be changed in the future.

If you want to run local Ethereum network you would need to install [ganache-cli](https://www.npmjs.com/package/ganache-cli) globally.

You would also need to install resources needed to compile nearcore (in the future this will only be required for the testing CLI):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup default stable
rustup target add wasm32-unknown-unknown
```

Then install dependencies needed for the compilation of nearcore, [see](https://docs.near.org/docs/local-setup/running-testnet#compiling-and-running-official-node-without-docker).

Then install golang, [see](https://golang.org/dl/).

We recommend using [nvm](https://github.com/nvm-sh/nvm) for installing node and npm, if you already don't have one.

## Usage
You can either install `rainbow-bridge-cli` from npm or clone it and use locally.
```bash
npm i -g rainbow-bridge-cli
```

To learn the commands that you can use with the rainbow bridge run
```bash
node index.js --help
```

Parameters of each command can be specified through environment variables, command line arguments, entries in the `~/.rainbow/config.json` config file, or the default value will be used -- in that priority.
If argument is not provided and there is no default value the program will not execute.

If script successfully executes a command then each parameter provided through the command line argument will be
written into the config file. Additionally, if scripts generates new parameters (e.g. it deploys a contract to Ethereum
and obtains its address) will also be written into the config file. Arguments should not be specified multiple times. 

Note, you can use environment variables to pass sensitive data which will not lead to it being written into the config file.

## Security

Bridge is as long as majority (1/2) of Etherem mining power is honest and supermajority (2/3) of NEAR stake is honest.
There are no additional security requirements, except that Ethereum should be able to accept 1 transaction within 4 hour period even in the worst congestion scenario.

## Gas costs

NEAR fees are negligible, both for bridge maintenance and for token transfer.
Ethereum fees are the following:
* To transfer ERC20 token from ETH to NEAR: Approx 43,989 gas to set allowance and approx 37,407 gas to lock it;
* To transfer ERC20 token back from NEAR to ETH: Approx 240,531 gas to unlock the token;
* To submit a NEAR block header: approx 697,140 gas;
* To challenge a NEAR block header: approx 700k gas.

As of 2020-07-14 (gas price is 40 gwei) the cost of running bridge on NEAR mainnnet and Ethereum mainnet is approx 42 USD/day. The cost of ETH->NEAR transfer of ERC20 token is 1 USD. The cost of NEAR->ETH transfer of ERC20 token is 2 USD.

## Using existing Testnet bridge

We are currently running bridge between NEAR Testnet and Ethereum Ropsten. The latency on it sped up to allow rapid experimentation.
Specifically, ETH->NEAR direction is 10 blocks which corresponds to 2-3 minutes latency, and NEAR->ETH direction is ~80 seconds.
The version of the bridge that will connect NEAR Mainnet to Ethereum Mainnet will use 25 blocks for ETH->NEAR direction and 4 hours for NEAR->ETH direction.

To try cross-blockchain transfer of an existing ERC20 token you would need:
* Account on Ropsten that owns certain amount of some ERC20 token, in this example we will be using `0x88f975D5A1153Ea92AF66e7c4292576a329c04B6`.
Ask max@near.org to transfer you some (if you want to transfer your own token you would have to setup your own locker and minting contracts);
* Account on https://wallet.testnet.near.org/;

Let's set them up:
1. Install [metamask](https://metamask.io/) and switch to Ropsten. Then create an account;
2. Go to https://faucet.ropsten.be/ and send some tokens to this account;
3. Add `0x88f975D5A1153Ea92AF66e7c4292576a329c04B6` as a custom token in Metamask (use `0` as precision);
4. Ask max@near.org to deposit you some ERC20 tokens. After the deposit you should be able to observe non-zero balance in Metamask;
5. Export private key from this account;
6. Go to https://wallet.testnet.near.org/ and create an account;
7. Install https://github.com/near/near-shell and run `near login`. In the opened browser tab/window allow new access key;
8. Record secret key that you have just added from `~/.near-credentials/default/<account name>.json`, include `ed25519:` prefix;
9. If you don't have access to an Ethereum node, go to http://infura.io/ and create a free account. Locate and record websocket URL to ropsten: `wss://ropsten.infura.io/ws/v3/<project_id>`;

Prepare CLI:
```bash
git clone https://github.com/near/rainbow-bridge/
cd rainbow-bridge/environment
yarn
node index.js clean
node index.js prepare
```
Then edit `~/.rainbow/config.json` to be:
```json
{
        "nearNetworkId": "testnet",
        "nearNodeUrl": "https://rpc.testnet.internal.near.org/",
        "ethNodeUrl": "wss://ropsten.infura.io/ws/v3/<project_id>",
        "nearMasterAccount": "<your_near_account_id>",
        "nearMasterSk": "<your_near_sk>",
        "eth2nearClientAccount": "eth2nearclient10",
        "eth2nearProverAccount": "eth2nearprover10",
        "ethMasterSk": "<your_eth_private_key>",
        "ethEd25519Address": "0x40b2d1334B7Fbbe2D4E1eb0Df689Af3D2a3903D2",
        "near2ethClientAddress": "0x276D4d74Dc14251c8D75Ff4aE9175142E1C2254d",
        "near2ethProverAddress": "0x69e75769a1D228f1c660869FB69455190fe9a80b",
        "ethErc20Address": "0x88f975D5A1153Ea92AF66e7c4292576a329c04B6",
        "nearFunTokenAccount": "funtoken10",
        "ethLockerAddress": "0xAfa909a33241d0271B7fd73b57C34439e9fBC84a",
}
```

Now let's run the transfer from ETH to NEAR:
```bash
node index.js transfer-eth-erc20-to-near --amount 100 --eth-sender-sk <you_eth_private_key> --near-receiver-account <your_near_account_id> --near-master-account <your_near_account_id> --near-master-sk <your_near_sk>
```
Wait until the command finishes. You should see something like:
```
P3oQ5ohqrayePpKy26RXdXPgUAD8DByeC5jhYA6kLJxTmXPd6gVHuP
Approving token transfer.
Approved token transfer.
Transferring tokens from the ERC20 account to the token locker account.
Success.
Transferring 100 tokens from 0x88f975D5A1153Ea92AF66e7c4292576a329c04B6 ERC20. From 0xe3628e6AB18A6B0F60Ed8540690d18b6d9C88a46 sender to flow10.testnet recipient
Eth2NearClient is currently at block 8292239. Waiting for block 8292241 to be confirmed. Sleeping for 10 sec.
Eth2NearClient is currently at block 8292241. Waiting for block 8292241 to be confirmed. Sleeping for 10 sec.
Eth2NearClient is currently at block 8292242. Waiting for block 8292241 to be confirmed. Sleeping for 10 sec.
Eth2NearClient is currently at block 8292243. Waiting for block 8292241 to be confirmed. Sleeping for 10 sec.
Eth2NearClient is currently at block 8292244. Waiting for block 8292241 to be confirmed. Sleeping for 10 sec.
Eth2NearClient is currently at block 8292246. Waiting for block 8292241 to be confirmed. Sleeping for 10 sec.
Eth2NearClient is currently at block 8292247. Waiting for block 8292241 to be confirmed. Sleeping for 10 sec.
Eth2NearClient is currently at block 8292248. Waiting for block 8292241 to be confirmed. Sleeping for 10 sec.
Eth2NearClient is currently at block 8292249. Waiting for block 8292241 to be confirmed. Sleeping for 10 sec.
Balance of flow10.testnet before the transfer is 0
TxHash Hm6eRyMCoLKRAkg7DMQuN4e25kLU4GUPLVFjD6PaWXPi
[ 'token: 88f975d5a1153ea92af66e7c4292576a329c04b6; sender: e3628e6ab18a6b0f60ed8540690d18b6d9c88a46; amount: 100; recipient: flow10.testnet',
  'Refunding 6300000000000000000000 tokens for storage' ]
Transferred
Balance of flow10.testnet after the transfer is 100
```
Currently, NEAR wallet does not display balances of the fungible token, but fortunately the CLI tool queries the balance before and after the transfer.

Now let's run the transfer from NEAR to ETH:
```bash
node index.js transfer-eth-erc20-from-near --amount 1 --near-sender-account <your_near_account_id> --near-sender-sk <your_near_sk> --eth-receiver-address <your_eth_address> --eth-master-sk <your_eth_private_key>
```
Wait until the command finishes. You should observe something like:
```
Balance of flow10.testnet before burning: 100
Burning 1 tokens on NEAR blockchain in favor of e3628e6AB18A6B0F60Ed8540690d18b6d9C88a46.
Current light client head is: hash=F2fkvdda1bwZiM5oMW1e5RLa5nLEGbrNQbFKJyCr63V6, height=9593826
Block 9593969 is not available on the light client yet. Current height of light client is 9593826. Sleeping 10 seconds.
Current light client head is: hash=GEqJfWRnNyvC6T7yJtzLrNxKqohVWX34La4xWZMeuHXy, height=9593959
Block 9593969 is not available on the light client yet. Current height of light client is 9593959. Sleeping 10 seconds.
Current light client head is: hash=GEqJfWRnNyvC6T7yJtzLrNxKqohVWX34La4xWZMeuHXy, height=9593959
Block 9593969 is not available on the light client yet. Current height of light client is 9593959. Sleeping 10 seconds.
Current light client head is: hash=GEqJfWRnNyvC6T7yJtzLrNxKqohVWX34La4xWZMeuHXy, height=9593959
Block 9593969 is not available on the light client yet. Current height of light client is 9593959. Sleeping 10 seconds.
Current light client head is: hash=GEqJfWRnNyvC6T7yJtzLrNxKqohVWX34La4xWZMeuHXy, height=9593959
Block 9593969 is not available on the light client yet. Current height of light client is 9593959. Sleeping 10 seconds.
Current light client head is: hash=GEqJfWRnNyvC6T7yJtzLrNxKqohVWX34La4xWZMeuHXy, height=9593959
Block 9593969 is not available on the light client yet. Current height of light client is 9593959. Sleeping 10 seconds.
Current light client head is: hash=GEqJfWRnNyvC6T7yJtzLrNxKqohVWX34La4xWZMeuHXy, height=9593959
Block 9593969 is not available on the light client yet. Current height of light client is 9593959. Sleeping 10 seconds.
Current light client head is: hash=GEqJfWRnNyvC6T7yJtzLrNxKqohVWX34La4xWZMeuHXy, height=9593959
Block 9593969 is not available on the light client yet. Current height of light client is 9593959. Sleeping 10 seconds.
Current light client head is: hash=GEqJfWRnNyvC6T7yJtzLrNxKqohVWX34La4xWZMeuHXy, height=9593959
Block 9593969 is not available on the light client yet. Current height of light client is 9593959. Sleeping 10 seconds.
Current light client head is: hash=GEqJfWRnNyvC6T7yJtzLrNxKqohVWX34La4xWZMeuHXy, height=9593959
Block 9593969 is not available on the light client yet. Current height of light client is 9593959. Sleeping 10 seconds.
Current light client head is: hash=GEqJfWRnNyvC6T7yJtzLrNxKqohVWX34La4xWZMeuHXy, height=9593959
Block 9593969 is not available on the light client yet. Current height of light client is 9593959. Sleeping 10 seconds.
Current light client head is: hash=GEqJfWRnNyvC6T7yJtzLrNxKqohVWX34La4xWZMeuHXy, height=9593959
Block 9593969 is not available on the light client yet. Current height of light client is 9593959. Sleeping 10 seconds.
Current light client head is: hash=GEqJfWRnNyvC6T7yJtzLrNxKqohVWX34La4xWZMeuHXy, height=9593959
Block 9593969 is not available on the light client yet. Current height of light client is 9593959. Sleeping 10 seconds.
Current light client head is: hash=GEqJfWRnNyvC6T7yJtzLrNxKqohVWX34La4xWZMeuHXy, height=9593959
Block 9593969 is not available on the light client yet. Current height of light client is 9593959. Sleeping 10 seconds.
Current light client head is: hash=GEqJfWRnNyvC6T7yJtzLrNxKqohVWX34La4xWZMeuHXy, height=9593959
Block 9593969 is not available on the light client yet. Current height of light client is 9593959. Sleeping 10 seconds.
Current light client head is: hash=72TJ5LrvpNyagTWPvEEiGHG84fyAWZCGwUjSXEe8mgLV, height=9594083
Near2EthClient block is at 9594083 which is further than the needed block 9593969
Burnt "1"
Balance of flow10.testnet after burning: 99
ERC20 balance of 0xe3628e6AB18A6B0F60Ed8540690d18b6d9C88a46 before the transfer: 99800
ERC20 balance of 0xe3628e6AB18A6B0F60Ed8540690d18b6d9C88a46 after the transfer: 99801
``` 
Congrats, you have completed a roundtrip of ERC20 tokens from ETH to NEAR and back to ETH!

## Local test run
To locally test the bridge run:
```bash
node index.js clean
node index.js prepare
node index.js start near-node
node index.js start ganache
```

### Initializing the contracts

First let's initialize the contracts that bridge needs to function:
```bash
node index.js init-near-contracts
node index.js init-eth-ed25519
node index.js init-near2eth-client --near2eth-client-lock-eth-amount 1000 --near2eth-client-lock-duration 10
node index.js init-near2eth-prover
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
node index.js start near-relay --eth-master-sk 0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501201
node index.js start near-watchdog --eth-master-sk 0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501202
```

Note, you can observe the logs of the relays by running:
```bash
pm2 logs
```

### Transferring tokens 
Finally, let's transfer some tokens 
```bash
node index.js transfer-eth-erc20-to-near --amount 1000 --eth-sender-sk 0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501200 --near-receiver-account eth2nearprover --near-master-account nearfuntoken
```
Note, when we deployed ERC20 to the Ethereum blockchain we have minted a large number of tokens to the default master
key of Ganache, so we have transferred ERC20 tokens from it to `alice.test.near`.
Notice that we are using `nearfuntoken` account here to pay for the NEAR gas fees, any account for which we know a secret key would've worked too.
You must observe blocks being submitted.

Now let's try to transfer one token back to Ethereum
```bash
node index.js transfer-eth-erc20-from-near --amount 1 --near-sender-account eth2nearprover --near-sender-sk ed25519:3D4YudUQRE39Lc4JHghuB5WM8kbgDDa34mnrEP5DdTApVH81af7e2dWgNPEaiQfdJnZq1CNPp5im4Rg5b733oiMP --eth-receiver-address 0xEC8bE1A5630364292E56D01129E8ee8A9578d7D8
```
You should observe the change of the ERC20 balance as reported by the CLI. 


# Docker:

## Currently we have the following docker options:

1. Rainbow Docker image containing rainbow ready for running
	- run the rainbow docker image with a custom command
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
docker-compose up

# Running the development env with ENV overrides 
docker-compose -f docker-compose-dev.yml up -e MASTER_SK=<key> -e ...

# Running the production env just use:
docker-compose -f docker-compose-prod.yml instead
```
