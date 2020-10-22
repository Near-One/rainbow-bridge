#!/usr/bin/env node

const path = require('path')
const fs = require('fs')

async function main() {
  let packageJson = require(path.join(__dirname, '../package.json'))
  if (process.env.PATCH_RAINBOW_BRIDGE_SOL) {
    packageJson.dependencies[
      'rainbow-bridge-sol'
    ] = `near/rainbow-bridge-sol#${process.env.PATCH_RAINBOW_BRIDGE_SOL}`
  }
  if (process.env.PATCH_RAINBOW_BRIDGE_RS) {
    packageJson.dependencies[
      'rainbow-bridge-rs'
    ] = `near/rainbow-bridge-rs#${process.env.PATCH_RAINBOW_BRIDGE_RS}`
  }
  if (process.env.PATCH_RAINBOW_BRIDGE_LIB) {
    packageJson.dependencies[
      'rainbow-bridge-lib'
    ] = `near/rainbow-bridge-lib#${process.env.PATCH_RAINBOW_BRIDGE_LIB}`
  }
  if (process.env.PATCH_TOKEN_CONNECTOR) {
    packageJson.dependencies[
      'rainbow-token-connector'
    ] = `near/rainbow-token-connector#${process.env.PATCH_TOKEN_CONNECTOR}`
  }
  console.log('Contract versions:')
  console.log(
    `rainbow-bridge-sol: ${packageJson.dependencies['rainbow-bridge-sol']}`
  )
  console.log(
    `rainbow-bridge-rs: ${packageJson.dependencies['rainbow-bridge-rs']}`
  )
  console.log(
    `rainbow-bridge-lib: ${packageJson.dependencies['rainbow-bridge-lib']}`
  )
  console.log(
    `rainbow-token-connector: ${packageJson.dependencies['rainbow-token-connector']}`
  )
  if (
    !process.env.PATCH_RAINBOW_BRIDGE_SOL &&
    !process.env.PATCH_RAINBOW_BRIDGE_RS &&
    !process.env.PATCH_RAINBOW_BRIDGE_LIB &&
    !process.env.PATCH_TOKEN_CONNECTOR
  ) {
    process.exit()
  }

  fs.writeFileSync(
    path.join(__dirname, '../package.json'),
    JSON.stringify(packageJson)
  )
}

main()
