# Rainbowup

Public scripts to launch rainbow bridge with all necessary services.

There are 3 separate use-cases:
* Production mode -- launching rainbow bridge in production. Only includes `ethrelay` and `nearrelay` services;
* Testing mode -- launches rainbow bridge in testing mode and runs tests on it. Includes `ethrelay`, `nearrelay` serives
as well as the local Near node and the Ganache client;
* Development mode -- same as testing mode but services are launched from the local code instead of being downloaded.


Note, currently it only starts the Near side of the bridge.

## Production
To run rainbowup run:
```bash
python3 main.py prepare
python3 main.py run --near_node_url=<Near node URL> --near_network_id=mainnet --near_master_key_path=<path to the keyfile> --eth_network=mainnet
```

To terminate the bridge run:
```bash
python3 main.py cleanup
```

## Testing
To run bridge without testing:
```bash
python3 main.py prepare
python3 run
```

To run bridge with testing:
```bash
python3 main.py prepare
python3 run
```

To cleanup local data:
```bash
python3 main.py cleanup
```

## Development
To start bridge using the local source:
```bash
python3 main.py prepare --source=<path to rainbow-bridge> --nearcore_source=<path to nearcore source>
python3 main.py run --source=<path to rainbow-bridge> --nearcore_source=<path to nearcore source>
```

To cleanup:
```bash
python3 main.py cleanup --source=<path to rainbow-bridge> --nearcore_source=<path to nearcore source>
```
