const ProcessManager = require('pm2')
const { spawnProcess } = require('./helpers')
const { Watchdog } = require('rainbow-bridge-watchdog')
const path = require('path')

class StartWatchdogCommand {
  static async execute ({
    daemon,
    ethNodeUrl,
    ethMasterSk,
    ethClientArtifactPath,
    ethClientAddress,
    watchdogDelay,
    watchdogErrorDelay,
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
        spawnProcess('bridge-watchdog', {
          name: 'bridge-watchdog',
          script: path.join(__dirname, '../../index.js'),
          interpreter: 'node',
          error_file: '~/.rainbow/logs/watchdog/err.log',
          out_file: '~/.rainbow/logs/watchdog/out.log',
          args: [
            'start',
            'bridge-watchdog',
            '--eth-node-url', ethNodeUrl,
            '--eth-master-sk', ethMasterSk,
            '--eth-client-artifact-path', ethClientArtifactPath,
            '--eth-client-address', ethClientAddress,
            '--watchdog-delay', watchdogDelay,
            '--watchdog-error-delay', watchdogErrorDelay,
            '--daemon', 'false',
            '--metrics-port', metricsPort
          ],
          logDateFormat: 'YYYY-MM-DD HH:mm:ss.SSS'
        })
      })
    } else {
      const watchdog = new Watchdog()
      await watchdog.initialize({
        ethNodeUrl,
        ethMasterSk,
        ethClientArtifactPath,
        ethClientAddress,
        metricsPort
      })
      await watchdog.run({
        ethMasterSk,
        ethClientAddress,
        watchdogDelay,
        watchdogErrorDelay
      })
    }
  }
}

exports.StartWatchdogCommand = StartWatchdogCommand
