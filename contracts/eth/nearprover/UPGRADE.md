# Upgradability

This file describes how to upgrade the bridge address in NearProver contract.

## Configuration

1. Create `.env` file inside `nearprover` directory: `$ touch .env`.

2. Add to the file your Infura API key:
`$ echo "INFURA_API_KEY=YOUR_INFURA_API_KEY_HERE" >> .env` <br/>

3. Add to the file Ethereum Private key (without '0x'):
`$ echo "ETH_PRIVATE_KEY=YOUR_ETH_PRIVATE_KEY_HERE" >> .env`

## Mock deployment

In order to have some testing prover contract instance for the testing, it could be useful to deploy some prover
instance to try upgradability.
To deploy the prover with the mock run: <br/>
`$ make deploy-prover-with-mock-bridge`

## Upgrade

1. First, you can fetch the current bridge address from the prover by running:<br/>
`$ make get-provers-bridge-address PROVER=<PROVER_ADDRESS_HERE>`

2. Upgrade the bridge address for the provided prover:<br/>
`$ make upgrade-provers-bridge-address-to PROVER=<PROVER_ADDRESS_HERE> NEW_BRIDGE=<BRIDGE_ADDRESS_HERE>`
or
`$ make upgrade-provers-bridge-address-to-with-ledger PROVER=<PROVER_ADDRESS_HERE> NEW_BRIDGE=<BRIDGE_ADDRESS_HERE> LEDGER_KEY_PATH=<LEDGER_KEY_PATH_HERE>`

3. Repeat the item 1 to ensure the Prover was updated with the new bridge address.

The default network is `goerli`. To execute the network you need to provide the `NETWORK` variable (goerli, ropsten, mainnet):
`$ make upgrade-provers-bridge-address-to PROVER=<PROVER_ADDRESS_HERE> NEW_BRIDGE=<BRIDGE_ADDRESS_HERE> NETWORK=mainnet`

## Upgrade Admin

1. Upgrade the contract's admin address for the provided admin:<br/>
`$ make upgrade-admin NETWORK=goerli ADDRESS=<CONTRACT_ADDRESS> CURRENT_ADMIN=<CURRENT_ADMIN_ADDRESS_HERE> NEW_ADMIN=<NEW_ADMIN_ADDRESS_HERE> SLOT=<SLOT_NUMBER_HERE>`
or
`$ make upgrade-admin-with-ledger NETWORK=goerli ADDRESS=<CONTRACT_ADDRESS> CURRENT_ADMIN=<CURRENT_ADMIN_ADDRESS_HERE> NEW_ADMIN=<NEW_ADMIN_ADDRESS_HERE> SLOT=<SLOT_NUMBER_HERE> LEDGER_KEY_PATH=<LEDGER_KEY_PATH_HERE>`
