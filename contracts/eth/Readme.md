# **_Near Bridge Upgradable_**
---

#### Near Bridge stores the block header from Near chain on ethereum side in trustless manner.

## To deploy Near-Bridge Upgradable:question:
* Setup `.env` file configurations as follows:-
  * `DEFENDER_TEAM_API_KEY`:- Your's hardhat defender API key.
  * `DEFENDER_TEAM_API_SECRET_KEY`: Your;s hardhat defender API secret key, obtained from defender account.
  * `ETHERSCAN_API_KEY`:- Etherscan API key to verify the contract automatically after deployments.
* **NOTE:** While deploying the network node URL is picked up from `'home/.rainbow/config.json'` directory, therefore this config must be setup before executing deployments.
  * For example: in config.json 
    * "`ethNodeUrl`" : "https://eth-goerli.g.alchemy.com/v2/RZeKGX8HziWu*******3GxgOMG1Zr2Yb",
	* "`ethMasterSk`" : "0xac0974bec39a17e36ba4a6b*******************************d7bf4f2ff80"
* run this command after all above setup:-
  * ```npx hardhat deployNearBridgeProxy --ethclientlockethamount <lock_eth_amount> --ethclientlockduration <lock_duration_in_seconds> --ethclientreplaceduration <replace_duration_in_nanoseconds> --ed25519 <ed25519_address> --pausedflags <pausedFlag (0 or 1)> --upgrader <gnosis_safe_address> --config rainbowBridgeConfig.js --network <network_name>```

## To transfer ownership of Near-Bridge Upgradable:question:
* **Note:** 
  * Current owner is the address who initially deployed the upgradable contract.
  * Only present owner of bridge can transfer their ownership to new owner. ie. `ethMasterSk` in `'home/.rainbow/config.json'` must be the pvt key of present owner.
* To transfer ownership run the following command:-
  * ```npx hardhat transferOwnership --currentadmin <present_owner_address> --newadmin <new_owner_address> --bridgeaddress <bridge_proxy_address> --config rainbowBridgeConfig.js --network <network_name>```

## To upgrade Near-Bridge-Upgradable from Multi-Sig safe:question:

* **Note:** Near-bridge can be upgraded by only multi-sig safe address provided as `upgrader` while deployment.
  * A proposal is created for upgradation through hardhat-defender where owners of safe can pass by signing threshold txn.
  * All m of n owner's in multi-sig must be present to sign txn from hardhat defender to upgrade the contract.
  * txn can be signed from defender after running the below command.
  * new implementation contract must be present in `./nearbridge/contracts` directory before running the command.
* Run this command to upgrade the Near_Bridge upgradable contract:-
  * ```npx hardhat proposeUpgrade --proxyaddress <near_bridge_proxy_address> --newcontractname <new_implementation_contract_name> --upgrader <multi_sig_address> --config rainbowBridgeConfig.js --network <network_name>```
---
# **_Near Prover Upgradable_**

#### Near Prover checks the integrity of Near blocks in Near bridge.

## To deploy Near-Bridge Upgradable❓

* Setup .env file configurations same as for Near bridge.
* Run this command after setup:-
  * ```npx hardhat deployNearProverProxy --ethclientaddress <near-bridge-eth-address> --pausedflags <paused Flag> --upgrader <multi-sig safe address>```
  
## To transfer ownership of Near-Prover Upgradable:question:
* **Note:** 
  * Current owner is the address who initially deployed the upgradable contract.
  * Only present owner of near prover can transfer their ownership to new owner. ie. `ethMasterSk` in `'home/.rainbow/config.json'` must be the pvt key of present owner.
* To transfer ownership run the following command:-
  * ```npx hardhat transferOwnership --currentadmin <present_owner_address> --newadmin <new_owner_address> --proveraddress <near_prover_proxy_address> --config rainbowBridgeConfig.js --network <network_name>```

## To upgrade Near-Prover-Upgradable from Multi-Sig safe:question:

* **Note:** Near-prover can be upgraded by only multi-sig safe address provided as `upgrader` while deployment.
  * A proposal is created for upgradation through hardhat-defender where owners of safe can pass by signing threshold txn.
  * All m of n owner's in multi-sig must be present to sign txn from hardhat defender to upgrade the contract.
  * txn can be signed from defender after running the below command.
  * new implementation contract must be present in `./nearprover/contracts` directory before running the command.
* Run this command to upgrade the Near_Prover upgradable contract:-
  * ```npx hardhat proposeUpgrade --proxyaddress <near_prover_proxy_address> --newcontractname <new_implementation_contract_name> --upgrader <multi_sig_address> --config rainbowBridgeConfig.js --network <network_name>```
---
  

