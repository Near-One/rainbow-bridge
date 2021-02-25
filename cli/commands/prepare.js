const { exec } = require('child_process')
const path = require('path')

const { getScript } = require('rainbow-bridge-utils')

class PrepareCommand {
  static execute ({ coreSrc }) {
    const scriptDir = getScript('prepare')

    const shell = ['bash', scriptDir].join(' ')

    const env = {}
    for (const e in process.env) {
      env[e] = process.env[e]
    }

    env.LOCAL_CORE_SRC = coreSrc && path.resolve(coreSrc)

    const prepareScript = exec(shell, { env: env })
    prepareScript.stdout.on('data', function (data) {
      process.stdout.write(data)
    })

    prepareScript.stderr.on('data', function (data) {
      process.stderr.write(data)
    })
  }
}

exports.PrepareCommand = PrepareCommand
