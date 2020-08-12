const ProcessManager = require('pm2')
const { spawnProcess } = require('./helpers')
const { Near2EthWatchdog } = require('../../lib/near2eth-watchdog')
const { RainbowConfig } = require('../../lib/config')
const path = require('path')
const os = require('os')

class StartNear2EthWatchdogCommand {
  static async execute() {
    if (RainbowConfig.getParam('daemon') === 'true') {
      ProcessManager.connect(err => {
        if (err) {
          console.log(
            'Unable to connect to the ProcessManager daemon! Please retry.'
          )
          return
        }
        spawnProcess('near-watchdog', {
          name: 'near-watchdog',
          script: path.join(__dirname, '../../index.js'),
          interpreter: 'node',
          error_file: '~/.rainbow/logs/near-watchdog/err.log',
          out_file: '~/.rainbow/logs/near-watchdog/out.log',
          args: ['start', 'near-watchdog', ...RainbowConfig.getArgsNoDaemon()],
          logDateFormat: 'YYYY-MM-DD HH:mm:ss.SSS',
        })
      })
    } else {
      const watchdog = new Near2EthWatchdog()
      await watchdog.initialize()
      await watchdog.run()
    }
  }
}

exports.StartNear2EthWatchdogCommand = StartNear2EthWatchdogCommand
