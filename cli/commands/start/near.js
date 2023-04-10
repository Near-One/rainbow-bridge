const { execSync } = require('child_process')
const request = require('request')

const { getLocalNearNodeURL } = require('./helpers')
const { readFileSync, writeFileSync } = require('fs')
const { homedir } = require('os')
const { join } = require('path')

const HOME = homedir()
const NEAR_BINARY_PATH = join(HOME, '.rainbow/core/target/debug')

class StartLocalNearNodeCommand {
  static execute ({ archival }) {
    const neardPath = join(NEAR_BINARY_PATH, 'neard')
    const localnetPath = join(HOME, '/.near/localnet')
    const initConfigCommand = `${neardPath} --home ${localnetPath} localnet  --v 1`
    const startNodeCommand = `nearup run localnet --num-nodes 1 --binary-path ${NEAR_BINARY_PATH}`

    request(getLocalNearNodeURL(), { json: true }, (err, _res, _body) => {
      if (err) {
        console.log(execSync(initConfigCommand).toString())

        if (archival === 'true') {
          const configPath = join(HOME, '.near/localnet/node0/config.json')
          const config = JSON.parse(readFileSync(configPath))
          config.archive = true
          writeFileSync(configPath, JSON.stringify(config))
        }

        console.log(execSync(startNodeCommand).toString())
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
