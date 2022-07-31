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
    ethClientArtifactPath,
    ethClientAddress,
    ethGasMultiplier,
    ethUseEip1559,
    near2ethRelayMinDelay,
    near2ethRelayMaxDelay,
    near2ethRelayErrorDelay,
    near2ethRelayBlockSelectDuration,
    near2ethRelayNextBlockSelectDelayMs,
    near2ethRelayAfterSubmitDelayMs,
    metricsPort,
    logVerbose
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
            '--eth-client-artifact-path', ethClientArtifactPath,
            '--eth-client-address', ethClientAddress,
            '--eth-gas-multiplier', ethGasMultiplier,
            '--eth-use-eip-1559', ethUseEip1559,
            '--near2eth-relay-min-delay', near2ethRelayMinDelay,
            '--near2eth-relay-max-delay', near2ethRelayMaxDelay,
            '--near2eth-relay-error-delay', near2ethRelayErrorDelay,
            '--near2eth-relay-block-select-duration', near2ethRelayBlockSelectDuration,
            '--near2eth-relay-next-block-select-delay-ms', near2ethRelayNextBlockSelectDelayMs,
            '--near2eth-relay-after-submit-delay-ms', near2ethRelayAfterSubmitDelayMs,
            '--daemon', 'false',
            '--metrics-port', metricsPort,
            '--log-verbose', logVerbose
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
        ethClientArtifactPath,
        ethClientAddress,
        ethGasMultiplier,
        metricsPort
      })
      await relay.run({
        near2ethRelayMinDelay,
        near2ethRelayMaxDelay,
        near2ethRelayErrorDelay,
        near2ethRelayBlockSelectDuration,
        near2ethRelayNextBlockSelectDelayMs,
        near2ethRelayAfterSubmitDelayMs,
        ethGasMultiplier,
        ethUseEip1559,
        logVerbose
      })
    }
  }
}

exports.StartNear2EthRelayCommand = StartNear2EthRelayCommand
