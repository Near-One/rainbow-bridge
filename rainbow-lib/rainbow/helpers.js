const nearlib = require('near-api-js')
const fs = require('fs')
const BN = require('bn.js')

const RETRY_NONCE = 10

// Check if account exists and if it does not creates it using master account. Also deploys the code and creates
// an access key.
async function maybeCreateAccount(
  near,
  masterAccountId,
  accountId,
  accountPK,
  initBalance,
  contractPath
) {
  if (!(await accountExists(near, accountId))) {
    console.log('Account %s does not exist creating it.', accountId)
    const masterAccount = new nearlib.Account(near.connection, masterAccountId)
    const balance = new BN(initBalance)
    let accountCreated = false
    for (let i = 0; i < RETRY_NONCE; i++) {
      try {
        await masterAccount.createAccount(accountId, accountPK, balance)
        accountCreated = true
        break
      } catch (e) {
        if (e.type && e.type === 'AccountAlreadyExists') {
          // Last createAccount can timeout, but actually success later
          accountCreated = true
          break
        }
        // retry on timeout, nonce error, and socket hangout
      }
    }
    if (!accountCreated) {
      console.log(
        `Failed to create account %s in ${RETRY_NONCE} retries due to nonce`,
        accountId
      )
      process.exit(1)
    }

    console.log('Created account %s', accountId)

    const account = new nearlib.Account(near.connection, accountId)

    let contractDeployed = false
    for (let i = 0; i < RETRY_NONCE; i++) {
      try {
        const data = fs.readFileSync(contractPath)
        await account.deployContract(data)
        contractDeployed = true
        break
      } catch (e) {
        if (e.message.includes('Transaction nonce')) {
          continue
        }
        console.log(
          'Failed to deploy contract to account %s. ERROR: %s',
          accountId,
          e
        )
        process.exit(1)
      }
    }
    if (!contractDeployed) {
      console.log(
        `Failed to deploy contract to account %s in ${RETRY_NONCE} retries due to nonce`,
        accountId
      )
      process.exit(1)
    }
    console.log('Deployed contract to account %s', accountId)
  }
}

// Checks whether the account exists.
async function accountExists(near, accountId) {
  const account = new nearlib.Account(near.connection, accountId)
  try {
    await account.fetchState()
    return true
  } catch (e) {
    return false
  }
}

// Checks whether the account has the key specified in the keyStore.
async function accountHasTheKey(near, accountId) {
  const account = new nearlib.Account(near.connection, accountId)
  const keyStoreKey = await near.config.deps.keyStore.getKey(
    near.config.networkId,
    accountId
  )
  const keys = await account.getAccessKeys()
  const accessKey = keys.find(
    (key) => key.public_key === keyStoreKey.getPublicKey().toString()
  )
  if (accessKey) {
    return true
  } else {
    return false
  }
}

// Verify that account exists and it has the key that we specified in the keyStore.
async function verifyAccount(near, accountId) {
  if (!(await accountExists(near, accountId))) {
    console.log(
      'Failed to fetch state of the %s account. Is it initialized?',
      accountId
    )
    process.exit(1)
  }

  if (!(await accountHasTheKey(near, accountId))) {
    console.log(
      'Account %s does not have the access key that can be used to operate with it.',
      accountId
    )
    process.exit(1)
  }
  return true
}

// Used in Status
async function verifyAccountGently(near, accountId) {
  if (!(await accountExists(near, accountId))) {
    return false
  }

  if (!(await accountHasTheKey(near, accountId))) {
    return false
  }
  // All checks have passed
  return true
}

exports.maybeCreateAccount = maybeCreateAccount
exports.accountExists = accountExists
exports.accountHasTheKey = accountHasTheKey
exports.verifyAccount = verifyAccount
exports.verifyAccountGently = verifyAccountGently
