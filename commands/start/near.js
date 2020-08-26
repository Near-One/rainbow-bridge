const util = require('util')
const { execSync } = require('child_process')
const request = require('request')
const { getLocalNearNodeURL } = require('./helpers')
const { RainbowConfig } = require('rainbow-bridge-lib/config')

class StartLocalNearNodeCommand {
  static execute() {
    const command = util.format(
      'python3 ~/.rainbow/nearup/main.py localnet --num-nodes 1 --binary-path %s',
      '~/.rainbow/core/target/debug'
    )
    request(getLocalNearNodeURL(), { json: true }, (err, _res, _body) => {
      if (err) {
        console.log(execSync(command).toString())
      } else {
        console.log('Local Node is already running. Skipping...')
      }
    })
    RainbowConfig.setParam('near-node-url', 'http://localhost:3030')
    RainbowConfig.setParam('near-network-id', 'local')
    RainbowConfig.setParam('near-master-account', 'node0')
    RainbowConfig.setParam(
      'near-master-sk',
      'ed25519:3D4YudUQRE39Lc4JHghuB5WM8kbgDDa34mnrEP5DdTApVH81af7e2dWgNPEaiQfdJnZq1CNPp5im4Rg5b733oiMP'
    )
    RainbowConfig.saveConfig()
  }
}

exports.StartLocalNearNodeCommand = StartLocalNearNodeCommand
