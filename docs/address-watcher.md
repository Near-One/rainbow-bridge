# Address monitoring

To monitor state of the accounts on NEAR and Ethereum side use `address-watcher` service. To start it run:

```
node cli/index.js start address-watcher --monitor-accounts-path monitor.json --metrics-port 8004
```

Accounts used by the bridge are monitored by default, to monitor extra accounts make sure to properly populate `monitor.json` file. Following is an example (`monitor.json` used for `ropsten` / `testnet` bridge):

```json
{
    "near": [
        {
            "id": "32c63ce502d72367c6cc1cf0d0e16f07675587ab.f290121.ropsten.testnet",
            "name": "erc20_on_near_default",
            "description": "erc20 on near using TToken"
        },
        {
            "id": "f290121.ropsten.testnet",
            "name": "near_token_factory",
            "description": "near token factory"
        }
    ],
    "ethereum": [
        {
            "address": "0xa5289b6d5dcc13e48f2cc6382256e51589849f86",
            "name": "eth_erc20_locker",
            "description": "ethereum erc20 locker"
        }
    ]
}
```
