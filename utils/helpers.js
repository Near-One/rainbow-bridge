const { nearAPI } = require('./robust')
const fs = require('fs')
const BN = require('bn.js')

const RETRY_NONCE = 10

// Check if account exists and if it does not creates it using master account. Also deploys the code and creates
// an access key.
async function maybeCreateAccount (
  near,
  masterAccountId,
  accountId,
  accountPK,
  initBalance,
  contractPath
) {
  const status = await getAccountStatus(near, accountId)

  if (status === accountStatus.ACCOUNT_DOES_NOT_EXIST) {
    console.log(`Account ${accountId} does not exist creating it.`)
    const masterAccount = new nearAPI.Account(near.connection, masterAccountId)
    const balance = new BN(initBalance)
    let accountCreated = false
    let lastError
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
        } else {
          // retry on timeout, nonce error, and socket hangout, not enough funds
          lastError = e
        }
      }
    }
    if (!accountCreated) {
      console.log(`Failed to create account ${accountId} in ${RETRY_NONCE} retries due to nonce`)
      console.error(lastError)
      process.exit(1)
    }

    console.log('Created account %s', accountId)
  } else {
    console.log(`Account ${accountId} already exist.`)
  }

  if (status !== accountStatus.WITH_CONTRACT) {
    const account = new nearAPI.Account(near.connection, accountId)

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

const EMPTY_HASH = '11111111111111111111111111111111'

const accountStatus = {
  ACCOUNT_DOES_NOT_EXIST: 'Account does not exist',
  EMPTY_ACCOUNT: 'No contract deployed',
  WITH_CONTRACT: 'With contract deployed'
}

async function getAccountStatus (near, accountId) {
  const account = new nearAPI.Account(near.connection, accountId)
  try {
    await account.fetchState()

    if (account._state.code_hash === EMPTY_HASH) {
      return accountStatus.EMPTY_HASH
    } else {
      return accountStatus.WITH_CONTRACT
    }
  } catch (e) {
    return accountStatus.ACCOUNT_DOES_NOT_EXIST
  }
}

// Checks whether the account exists.
async function accountExists (near, accountId) {
  const account = new nearAPI.Account(near.connection, accountId)
  try {
    await account.state()
    return true
  } catch (e) {
    console.log(e)
    return false
  }
}

// Checks whether the account has the key specified in the keyStore.
async function accountHasTheKey (near, accountId) {
  const account = new nearAPI.Account(near.connection, accountId)
  const keyStoreKey = await near.config.keyStore.getKey(
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
async function verifyAccount (near, accountId) {
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
async function verifyAccountGently (near, accountId) {
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
