const ProcessManager = require('pm2')
const util = require('util')

const NEAR_PORT = 3030
const GANACHE_PORT = 9545

function spawnProcess (name, config) {
  ProcessManager.describe(name, (err, process) => {
    if (err) {
      console.log(
        'Unable to list all the process running by the ProcessManager daemon!'
      )
    }
    if (process.length) {
      console.log('%s service is running...', name)
      ProcessManager.disconnect()
    } else {
      console.log('%s service is not running...', name)
      console.log('Starting %s with command:', name)
      console.log('%s %s', config.script, config.args)
      ProcessManager.start(config, (err, _proc) => {
        console.log('%s started...', name)
        if (!err) {
          console.log('Disconnecting from daemon')
        } else {
          console.log(err)
        }
        ProcessManager.disconnect()
      })
    }
  })
}

function getLocalNearNodeURL (port = NEAR_PORT) {
  return util.format('http://localhost:%d', port)
}

function getLocalGanacheNodeURL (port = GANACHE_PORT) {
  return util.format('http://localhost:%d', port)
}

exports.NEAR_PORT = NEAR_PORT
exports.GANACHE_PORT = GANACHE_PORT
exports.spawnProcess = spawnProcess
exports.getLocalNearNodeURL = getLocalNearNodeURL
exports.getLocalGanacheNodeURL = getLocalGanacheNodeURL
