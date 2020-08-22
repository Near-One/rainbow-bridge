// TODO get rid of it completely
//
// This shitty code appeared because the author haven't found
// the proper way to import binary files from npm packages to js code directly.
//
// This code is for demonstration only and must be rewritten.

const fs = require('fs')

function GetAbi(source, contract) {
  const abiPath =
    './node_modules/rainbow-bridge-sol/' +
    source +
    '/dist/' +
    contract +
    '.full.abi'
  return JSON.parse(fs.readFileSync(abiPath))
}

function GetBin(source, contract) {
  const binPath =
    './node_modules/rainbow-bridge-sol/' +
    source +
    '/dist/' +
    contract +
    '.full.bin'
  return '0x' + fs.readFileSync(binPath)
}

exports.GetAbi = GetAbi
exports.GetBin = GetBin
