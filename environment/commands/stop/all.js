const ProcessManager = require('pm2-promise')

const { StopLocalNearNodeCommand } = require('./near.js')

async function stop(serviceName) {
  try {
    await ProcessManager.delete(serviceName)
    console.log(serviceName, 'successfully stopped...')
  } catch (err) {
    if (!err.message.includes('process or namespace not found')) {
      console.log(serviceName, 'error stopping the process due to:', err)
    } else {
      console.log(serviceName, 'already stopped')
    }
  }
}

class StopAllCommands {
  static async execute() {
    console.log('Stopping all processes...')

    StopLocalNearNodeCommand.execute()
    for (const serviceName of [
      'ganache',
      'eth-relay',
      'near-relay',
      'near-watchdog',
    ]) {
      await stop(serviceName)
    }

    console.log('Stopping all processes done')
    await ProcessManager.disconnect()
  }
}

exports.StopAllCommands = StopAllCommands
