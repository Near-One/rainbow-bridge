const { exec } = require('child_process')
const path = require('path')
const { RainbowConfig, getScript } = require('rainbow-bridge-utils')

class PrepareCommand {
  static execute () {
    const scriptDir = getScript('prepare')

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
    const prepareScript = exec(shell, { env: env })
    // @ts-ignore
    prepareScript.stdout.on('data', function (data) {
      process.stdout.write(data)
    })

    // @ts-ignore
    prepareScript.stderr.on('data', function (data) {
      process.stderr.write(data)
    })
  }
}

exports.PrepareCommand = PrepareCommand
