const { execSync } = require('child_process')
const ProcessManager = require('pm2')
const { existsSync } = require('fs')
const { homedir } = require('os')
const path = require('path')

class CleanCommand {
  static execute() {
    console.log('Stopping all the running processes...')
    ProcessManager.connect((err) => {
      if (err) {
        // Never happened, but just log in case
        console.log('Failed to launch pm2 daemon')
      }
      ProcessManager.killDaemon((err) => {
        if (err) {
          console.log(`Error stopping pm2 processes. ${err}`)
          process.exit(1)
        }
        ProcessManager.disconnect()
        if (existsSync(path.join(homedir(), '.rainbow', 'nearup', 'main.py'))) {
          try {
            console.log('Stopping nearup')
            execSync('python3 ~/.rainbow/nearup/main.py stop')
          } catch (err) {
            console.log(`Error stopping nearup ${err}`)
          }
        }
        console.log('Cleaning ~/.rainbow and ~/.near/localnet directory...')
        execSync('rm -rf ~/.rainbow ~/.near/localnet')
        execSync('rm -f /tmp/near2ethtransfer.out /tmp/eth2neartransfer.out')
        console.log('Cleaning done...')
        process.exit(0)
      })
    })
  }
}

exports.CleanCommand = CleanCommand
