# Development

The loosely coupled architecture of the Rainbow Bridge allows its parts to be developed independently of each other.
This document describes the development workflow for each such component.

## Preparation

This assumes that supported versions of the `nvm`, `yarn`, `hardhat` are already installed.

Switch to a supported `node` version and install dependencies:
```bash
nvm install 13
yarn install
```
## Near2Eth Relay

The service responsible for submitting the NEAR Light Client block  to Ethereum smart contracts.

Compile Solidity contracts:
```bash
cd contracts/eth/nearbridge
yarn install
yarn build
```

Run local Hardhat node:
```bash
hardhat node
```

Deploy ED25519 Solidity contract:
```bash
cli/index.js init-eth-ed25519 \
    --eth-node-url http://127.0.0.1:8545/ \
    --eth-master-sk 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
```
Address of deployed contract: `0x5fbdb2315678afecb367f032d93f642f64180aa3`

**NOTE:** The private key specified here is the key of the first account created by [Hardhat](https://hardhat.org/hardhat-network/#running-stand-alone-in-order-to-support-wallets-and-other-software) and is publicly known.

Deploy and initialize EthClient contracts:
```bash
cli/index.js init-eth-client \
    --eth-node-url http://127.0.0.1:8545/ \
    --eth-master-sk 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
    --eth-ed25519-address 0x5fbdb2315678afecb367f032d93f642f64180aa3 \
    --eth-client-lock-duration 30 \
    --eth-client-replace-duration 60
```
Address of deployed contract: `0xe7f1725e7734ce288f8367e1bb143e90bb3f0512`

Start Near2EthRelay:
```bash
cli/index.js start near2eth-relay \
    --eth-node-url http://127.0.0.1:8545/ \
    --eth-master-sk 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
    --near-node-url https://rpc.testnet.near.org/ \
    --near-network-id testnet \
    --eth-client-address 0xe7f1725e7734ce288f8367e1bb143e90bb3f0512 \
    --eth-use-eip-1559 true \
    --near2eth-relay-max-delay 10 \
    --near2eth-relay-block-select-duration 30 \
    --near2eth-relay-after-submit-delay-ms 1000 \
    --log-verbose true \
    --daemon false
```
