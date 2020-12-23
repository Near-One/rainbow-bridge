'use strict'

const { readFileSync } = require('fs')

const Web3 = require('web3')
const ganache = require('ganache-cli')

const abi = JSON.parse(readFileSync('Verifier.abi'))
const code = readFileSync('Verifier.bin', { encoding: 'utf8' })
const tests = JSON.parse(readFileSync('tests.json'))

;(async () => {
  try {
    const web3 = new Web3(ganache.provider({ gasLimit: 1e9 }))
    const addr = (await web3.eth.personal.getAccounts())[0]

    const contract = await new web3.eth.Contract(abi).deploy({ data: code }).send({ from: addr, gas: 1e7 })
    let i = 0
    for (const test of tests) {
      console.error(`Test ${i}, uid = ${test.uid}`)
      await contract.methods.setRoot(`0x${test.root}`).send({ from: addr, handleRevert: true })
      const status = await contract.methods.verify(`0x${test.sender}`, `0x${test.message}`, test.uid, `0x${test.proof}`)
        .send({ from: addr, handleRevert: true })
      const events = []
      for (let i = 0; status.events[i]; i++) {
        events.push(status.events[i])
      }
      if (events.length && !events[events.length - 1].raw.topics.length) {
        for (const e of events) {
          console.error(e)
        }
        throw new Error('Something is wrong')
      }
      i++
    }
  } catch (e) {
    console.error(e)
    process.exit(1)
  }
})()
