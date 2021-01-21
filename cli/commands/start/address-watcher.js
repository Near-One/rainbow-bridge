const ProcessManager = require('pm2')
const { spawnProcess } = require('./helpers')
const { AddressWatcher } = require('../../../utils/address-watcher')
const path = require('path')

class AddressWatcherCommand {
  static async execute ({
    ethNodeUrl,
    nearNodeUrl,
    nearNetworkId,
    ethMasterSk,
    nearClientAccount,
    nearMasterAccount,
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
            '--daemon', 'false',
            '--metrics-port', metricsPort
          ],
          logDateFormat: 'YYYY-MM-DD HH:mm:ss.SSS'
        })
      })
    } else {
      const watcher = new AddressWatcher()

      const nearAccounts = [
        { id: nearClientAccount, name: 'near_client_account', description: 'eth on near client account' },
        { id: nearMasterAccount, name: 'near_relayer_account', description: 'near relayer account' }
      ]

      const ethereumAccounts = [
        { sk: ethMasterSk, name: 'ethereum_master_account', description: 'ethereum master sk' }
      ]

      await watcher.initialize({ ethNodeUrl, nearNodeUrl, nearNetworkId, nearAccounts, ethereumAccounts, metricsPort })
      await watcher.run()
    }
  }
}

exports.AddressWatcherCommand = AddressWatcherCommand
