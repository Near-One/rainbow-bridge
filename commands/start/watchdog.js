const ProcessManager = require('pm2')
const { spawnProcess } = require('./helpers')
const { Watchdog } = require('rainbow-bridge-lib/watchdog')
const { RainbowConfig } = require('rainbow-bridge-lib/config')
const path = require('path')
const os = require('os')

class StartWatchdogCommand {
  static async execute() {
    if (RainbowConfig.getParam('daemon') === 'true') {
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
            ...RainbowConfig.getArgsNoDaemon(),
          ],
          logDateFormat: 'YYYY-MM-DD HH:mm:ss.SSS',
        })
      })
    } else {
      const watchdog = new Watchdog()
      await watchdog.initialize()
      await watchdog.run()
    }
  }
}

exports.StartWatchdogCommand = StartWatchdogCommand
