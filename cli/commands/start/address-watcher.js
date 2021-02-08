const ProcessManager = require('pm2')
const { spawnProcess } = require('./helpers')
const { AddressWatcher } = require('../../../utils/address-watcher')
const path = require('path')
const fs = require('fs')
const Web3 = require('web3')

class AddressWatcherCommand {
  static async execute ({
    ethNodeUrl,
    nearNodeUrl,
    nearNetworkId,
    ethMasterSk,
    nearClientAccount,
    nearMasterAccount,
    monitorAccountsPath,
    daemon,
    metricsPort
  }) {
    if (daemon === 'true') {
      ProcessManager.connect((err) => {
        if (err) {
          console.log(
            'Unable to connect to the ProcessManager daemon! Please retry.'
          )
          return
        }
        spawnProcess('address-watcher', {
          name: 'address-watcher',
          script: path.join(__dirname, '../../index.js'),
          interpreter: 'node',
          error_file: '~/.rainbow/logs/address-watcher/err.log',
          out_file: '~/.rainbow/logs/address-watcher/out.log',
          args: [
            'start',
            'address-watcher',
            '--eth-node-url', ethNodeUrl,
            '--near-node-url', nearNodeUrl,
            '--near-network-id', nearNetworkId,
            '--eth-master-sk', ethMasterSk,
            '--near-client-account', nearClientAccount,
            '--near-master-account', nearMasterAccount,
            '--monitor-accounts-path', monitorAccountsPath,
            '--daemon', 'false',
            '--metrics-port', metricsPort
          ],
          logDateFormat: 'YYYY-MM-DD HH:mm:ss.SSS'
        })
      })
    } else {
      const watcher = new AddressWatcher()
      const web3 = new Web3(ethNodeUrl)

      const nearAccounts = [
        { id: nearClientAccount, name: 'near_client_account', description: 'eth on near client account' },
        { id: nearMasterAccount, name: 'near_relayer_account', description: 'near relayer account' }
      ]

      const ethereumAccounts = [
        { address: web3.eth.accounts.privateKeyToAccount(ethMasterSk).address, name: 'ethereum_master_account', description: 'ethereum master sk' }
      ]

      if (monitorAccountsPath !== '') {
        // Load other accounts to be monitored too.
        // By default only accounts directly related to the bridge are monitored.
        // No accounts related to token connector or other services are monitored.
        const monitorList = JSON.parse(fs.readFileSync(monitorAccountsPath))
        monitorList.near.forEach((nearAccount) => { nearAccounts.push(nearAccount) })
        monitorList.ethereum.forEach((ethereumAccount) => { ethereumAccounts.push(ethereumAccount) })
      }

      await watcher.initialize({ ethNodeUrl, nearNodeUrl, nearNetworkId, nearAccounts, ethereumAccounts, metricsPort })
      await watcher.run()
    }
  }
}

exports.AddressWatcherCommand = AddressWatcherCommand
