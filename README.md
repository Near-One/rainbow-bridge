# Bridge

[![Build Status](https://travis-ci.org/nearprotocol/near-bridge.svg?branch=master)](https://travis-ci.org/nearprotocol/near-bridge)
[![Coverage Status](https://coveralls.io/repos/github/nearprotocol/near-bridge/badge.svg)](https://coveralls.io/github/nearprotocol/near-bridge)

## Description

**Near <> Ethereum** decentralized bridge

Start with wiki: https://github.com/nearprotocol/bridge/wiki

The Ethereum<>NEAR bridge is going to be implemented as mutual smart contract based light client, thus would provide full security of respective networks for the bridge. Currentlty, building NEAR smart contract and relayer for Etheruem blockchain light verification.

## Projects list

- **EthBridge**: Rust (WASM) smart contract for Near blockchain, Ethereum light client storing hashes of blocks
- **EthRelay**: NodeJS application, streaming Ethereum block headers to **EthBridge** smart contract in Near blockchain.
- **EthProver**: Rust (WASM) smart contract for Near blockchain, helps verify log entry was included in tx receipt, which was included in block
- **NearBridge**: Solidity smart contractfor for Ethereum blockchain, Near light client stoging hashes of blocks
- **NearRelay**: NodeJS application, streaming Near block headers to **NearBridge** smart contract in Ethereum blockchain.
- **NearProver**: Solidity smart contractfor Ethereum blockchain, helps verify tx outcome was included in in block

