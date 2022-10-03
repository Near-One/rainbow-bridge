# Watchdog configuration for Mainnet

### Prerequisites

* NodeJS (v14+)
* npm/yarn

### Initial setup

Clone `rainbow-bridge` GitHub repo:
```
$ git clone https://github.com/aurora-is-near/rainbow-bridge/
```

Prepare an environment:
```
$ cd rainbow-bridge
$ yarn
$ mkdir -p ~/.rainbow
```

Build contracts:
```
$ cd contracts/eth/nearbridge
$ yarn
$ yarn build
$ cd -
```

### Rainbow Bridge config

Create and fill `~/.rainbow/config.json` file. The content of the config file for running watchdog should be the following:

```json=
{
        "ethNodeUrl": "",
        "ethClientAddress": "0x3FEFc5A4B1c02f21cBc8D3613643ba0635b9a873",
        "ethMasterSk": "",
        "watchdogDelay": 600
}
```

Please fill `ethNodeUrl` (any web3 node, e.g. [Infura](https://infura.io), [Alchemy](https://www.alchemy.com), etc) and `ethMasterSk` (private key for the expected watchdog EOA)

### Start the watchdog

```
$ cli/index.js start bridge-watchdog
```

### Check whether the watchdog is running

Ensure the watchdog is in the list of processes in `pm2` and running:
```
$ pm2 list
```

Check `pm2` logs for the watchdog
```
$ pm2 logs bridge-watchdog
```


## Troubleshooting

If you have any trouble running the watchdog, one of the issues might be because you don't have `pm2` installed.

So to fix this:
* either install a `pm2` following the [instruction](#Installing-pm2) and try again (recommended)
* or run the watchdog without daemon mode (not-recommended):
```$ cli/index.js start bridge-watchdog --daemon false```

### Additional info

To get more info on each of the parameters, run:

```
$ cli/index.js start bridge-watchdog --help
```

And get as a response an output similar to this:
```
Usage: index start bridge-watchdog [options]

Options:
  --eth-node-url <eth_node_url>                          The URL of the Ethereum node. (default: "")
  --eth-master-sk <eth_master_sk>                        The secret key of the master account on Ethereum blockchain. (default: "")
  --eth-client-artifact-path <eth_client_artifact_path>  Path to the artifact file defining Ethereum Client contract. (default: "/home/username/src/rainbow-bridge/contracts/eth/nearbridge/artifacts/contracts/NearBridge.sol/NearBridge.json")
  --eth-client-address <eth_client_address>              ETH address of the EthClient contract.
  --watchdog-delay <watchdog_delay>                      Number of seconds to wait after validating all signatures. (default: 600)
  --watchdog-error-delay <watchdog_error_delay>          Number of seconds to wait before retrying if there is an error. (default: "1")
  --daemon <daemon>                                      Whether the process should be launched as a daemon. (default: "true")
  --metrics-port <metrics_port>                          On which port to expose metrics for corresponding relayer, if not provided no metrics exposed
```

### Installing `pm2`

With `yarn`:
```
$ yarn global add pm2
```

With `npm`:

```
$ npm install pm2 -g
```

With _debian_, use the install script:

```
$ apt update && apt install sudo curl && curl -sL https://raw.githubusercontent.com/Unitech/pm2/master/packager/setup.deb.sh | sudo -E bash -
```
