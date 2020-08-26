const { exec } = require('child_process')
const path = require('path')
const { RainbowConfig } = require('rainbow-bridge-lib/config')

class PrepareCommand {
  static execute() {
    var scriptDir = path.resolve(__dirname, '../scripts/prepare.sh')

    const shell = ['bash', scriptDir].join(' ')

    const env = {}
    for (const e in process.env) {
      env[e] = process.env[e]
    }

    env.LOCAL_CORE_SRC =
      RainbowConfig.getParam('core-src') &&
      path.resolve(RainbowConfig.getParam('core-src'))
    env.LOCAL_NEARUP_SRC =
      RainbowConfig.getParam('nearup-src') &&
      path.resolve(RainbowConfig.getParam('nearup-src'))

    // @ts-ignore
    var prepareScript = exec(shell, { env: env })
    // @ts-ignore
    prepareScript.stdout.on('data', function (data) {
      console.log(data.toString())
    })
    // @ts-ignore
    prepareScript.stderr.on('data', function (data) {
      console.log(data.toString())
    })
  }
}

exports.PrepareCommand = PrepareCommand
