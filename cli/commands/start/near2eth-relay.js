const ProcessManager = require('pm2')
const { spawnProcess } = require('./helpers')
const { Near2EthRelay } = require('rainbow-bridge-near2eth-block-relay')
const path = require('path')

class StartNear2EthRelayCommand {
  static async execute ({
    daemon,
    nearNodeUrl,
    nearNetworkId,
    ethNodeUrl,
    ethMasterSk,
    ethClientAbiPath,
    ethClientAddress,
    ethGasMultiplier,
    near2ethRelayMinDelay,
    near2ethRelayMaxDelay,
    near2ethRelayErrorDelay,
    metricsPort
  }) {
    if (daemon === 'true') {
      ProcessManager.connect((err) => {
        if (err) {
          console.log(
            'Unable to connect to the ProcessManager daemon! Please retry.'
          )
          return
        }
        spawnProcess('near2eth-relay', {
          name: 'near2eth-relay',
          script: path.join(__dirname, '../../index.js'),
          interpreter: 'node',
          error_file: '~/.rainbow/logs/near2eth-relay/err.log',
          out_file: '~/.rainbow/logs/near2eth-relay/out.log',
          args: [
            'start', 'near2eth-relay',
            '--near-node-url', nearNodeUrl,
            '--near-network-id', nearNetworkId,
            '--eth-node-url', ethNodeUrl,
            '--eth-master-sk', ethMasterSk,
            '--eth-client-abi-path', ethClientAbiPath,
            '--eth-client-address', ethClientAddress,
            '--eth-gas-multiplier', ethGasMultiplier,
            '--near2eth-relay-min-delay', near2ethRelayMinDelay,
            '--near2eth-relay-max-delay', near2ethRelayMaxDelay,
            '--near2eth-relay-error-delay', near2ethRelayErrorDelay,
            '--daemon', 'false',
            '--metrics-port', metricsPort
          ],
          wait_ready: true,
          kill_timeout: 60000,
          logDateFormat: 'YYYY-MM-DD HH:mm:ss.SSS'
        })
      })
    } else {
      const relay = new Near2EthRelay()
      await relay.initialize({
        nearNodeUrl,
        nearNetworkId,
        ethNodeUrl,
        ethMasterSk,
        ethClientAbiPath,
        ethClientAddress,
        ethGasMultiplier,
        metricsPort
      })
      await relay.run({
        near2ethRelayMinDelay,
        near2ethRelayMaxDelay,
        near2ethRelayErrorDelay,
        ethGasMultiplier
      })
    }
  }
}

exports.StartNear2EthRelayCommand = StartNear2EthRelayCommand
