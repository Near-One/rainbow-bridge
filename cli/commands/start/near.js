const util = require('util')
const { execSync } = require('child_process')
const request = require('request')

const { getLocalNearNodeURL } = require('./helpers')

class StartLocalNearNodeCommand {
  static execute () {
    const command = util.format(
      'nearup run localnet --num-nodes 1 --binary-path %s',
      '~/.rainbow/core/target/debug'
    )
    request(getLocalNearNodeURL(), { json: true }, (err, _res, _body) => {
      if (err) {
        console.log(execSync(command).toString())
      } else {
        console.log('Local Node is already running. Skipping...')
      }
    })
    return {
      nearNodeUrl: 'http://localhost:3030',
      nearNetworkId: 'local',
      nearMasterAccount: 'node0',
      nearMasterSk: 'ed25519:3D4YudUQRE39Lc4JHghuB5WM8kbgDDa34mnrEP5DdTApVH81af7e2dWgNPEaiQfdJnZq1CNPp5im4Rg5b733oiMP'
    }
  }
}

exports.StartLocalNearNodeCommand = StartLocalNearNodeCommand
