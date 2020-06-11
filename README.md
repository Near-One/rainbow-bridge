<div align="center">

  <h1><code>Rainbow Bridge</code></h1>

  <p>
    <strong>Ethereum to Near trustless, fully decentralized, bidirectional bridge</strong>
  </p>
  
  <p>
    <a href="https://travis-ci.com/near/rainbow-bridge"><img src="https://travis-ci.com/near/rainbow-bridge.svg?branch=master" alt="Travis Build" /></a>
  </p>

## Local test run

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
