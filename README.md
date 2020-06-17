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

Parameters of each command can be specified through environment variables, command line arguments, entries in the `~/.rainbowup/config.js` config file, or the default value will be used -- in that priority.
If argument is not provided and there is no default value the program will not execute.

If script successfully executes a command then each parameter provided through the command line argument will be
written into the config file. Additionally, if scripts generates new parameters (e.g. it deploys a contract to Ethereum
and obtains its address) will also be written into the config file. Arguments should not be specified multiple times. 

Note, you can use environment variables to pass sensitive data which will not lead to it being written into the config file.


## Examples

### Local test run

To locally test the bridge run:
```bash
node index.js clean
node index.js prepare
node index.js start near-node
node index.js start near-node
node index.js start ganache
node index.js init-near-contracts --near-node-url http://localhost:3030 --near-network-id local --master-account node0 --master-sk ed25519:3D4YudUQRE39Lc4JHghuB5WM8kbgDDa34mnrEP5DdTApVH81af7e2dWgNPEaiQfdJnZq1CNPp5im4Rg5b733oiMP --client-account eth2nearclient --client-sk ed25519:3D4YudUQRE39Lc4JHghuB5WM8kbgDDa34mnrEP5DdTApVH81af7e2dWgNPEaiQfdJnZq1CNPp5im4Rg5b733oiMP --client-contract-path /Users/maksymzavershynskyi/Projects/rainbow-bridge/libs-rs/res/eth_client.wasm --client-init-balance 100000000000000000000000000 --validate-ethash false --prover-account eth2nearprover --prover-sk ed25519:3D4YudUQRE39Lc4JHghuB5WM8kbgDDa34mnrEP5DdTApVH81af7e2dWgNPEaiQfdJnZq1CNPp5im4Rg5b733oiMP --prover-contract-path /Users/maksymzavershynskyi/Projects/rainbow-bridge/libs-rs/res/eth_prover.wasm --prover-init-balance 100000000000000000000000000
node index.js start eth-relay --master-account node0 --master-sk ed25519:3D4YudUQRE39Lc4JHghuB5WM8kbgDDa34mnrEP5DdTApVH81af7e2dWgNPEaiQfdJnZq1CNPp5im4Rg5b733oiMP --client-account eth2nearclient --eth-node-url ws://localhost:9545 --near-node-url http://localhost:3030 --near-network-id local --daemon true
```
Please update the local paths to your file system.
Also, note that there is a bug which requires running `near-node` twice.

Then run
```bash
pm2 logs
```
You must observe blocks being submitted.
