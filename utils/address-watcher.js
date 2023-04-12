const Web3 = require('web3')
const BN = require('bn.js')
const nearAPI = require('near-api-js')
const { HttpPrometheus } = require('./http-prometheus.js')
const { sleep } = require('./robust.js')

class AddressWatcher {
  async initialize ({
    ethNodeUrl,
    nearNodeUrl,
    networkId,
    nearAccounts,
    ethereumAccounts,
    metricsPort
  }) {
    this.web3 = new Web3(ethNodeUrl)
    this.near = await nearAPI.connect({
      nodeUrl: nearNodeUrl,
      networkId: networkId,
      keyStore: new nearAPI.keyStores.InMemoryKeyStore()
    })

    this.metricsPort = metricsPort
    this.httpPrometheus = new HttpPrometheus(this.metricsPort, 'near_bridge_address_watcher_')

    this.nearAccounts = await Promise.all(nearAccounts.map(async (nearAccount) => {
      nearAccount.balanceGauge = this.httpPrometheus.gauge(nearAccount.name + '_balance_nano_near', nearAccount.description + ' balance in nano near')
      nearAccount.stateStorageGauge = this.httpPrometheus.gauge(nearAccount.name + '_storage_bytes', nearAccount.description + ' storage in bytes')
      console.log('Watching NEAR account:', nearAccount.id, nearAccount.name)
      return nearAccount
    }))

    this.ethereumAccounts = ethereumAccounts.map((ethAccount) => {
      ethAccount.balanceGauge = this.httpPrometheus.gauge(ethAccount.name + '_balance_gwei', ethAccount.description + ' balance in gwei')
      console.log('Watching Ethereum account:', ethAccount.address, ethAccount.name)
      return ethAccount
    })

    console.log('Address watcher started')
  }

  async run () {
    const nearYocto2Nano = new BN(10).pow(new BN(15))
    const ethWei2Gwei = new BN(10).pow(new BN(9))

    while (true) {
      await Promise.all(this.nearAccounts.map(async (nearAccount) => {
        try {
          const account = await this.near.account(nearAccount.id)
          const state = await account.state()
          const balanceNanoNear = new BN(state.amount).div(nearYocto2Nano).toNumber()
          const storageBytes = new BN(state.storage_usage).toNumber()
          nearAccount.balanceGauge.set(balanceNanoNear)
          nearAccount.stateStorageGauge.set(storageBytes)
        } catch (err) {
          console.log(err)
        }
      }).concat(this.ethereumAccounts.map(async (ethereumAccount) => {
        try {
          const balance = await this.web3.eth.getBalance(ethereumAccount.address)
          const balanceGwei = new BN(balance).div(ethWei2Gwei).toNumber()
          ethereumAccount.balanceGauge.set(balanceGwei)
        } catch (err) {
          console.error(err)
        }
      })))

      await sleep(1_000)
    }
  }
}

exports.AddressWatcher = AddressWatcher
