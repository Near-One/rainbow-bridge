const fs = require('fs').promises
const Path = require('path')
const fetch = require('node-fetch')

async function getLatestBlock (nearNodeUrl) {
  const resp = await fetch(nearNodeUrl, {
    method: 'post',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      jsonrpc: '2.0',
      id: 'dontcare',
      method: 'block',
      params: {
        finality: 'final'
      }
    })
  })
  const data = await resp.json()
  return data.result
}

async function getBlockChunk (nearNodeUrl, block) {
  const resp = await fetch(nearNodeUrl, {
    method: 'post',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      jsonrpc: '2.0',
      id: 'dontcare',
      method: 'chunk',
      params: [block.chunks[0].chunk_hash]
    })
  })
  const data = await resp.json()
  return data.result
}

async function getTxProof (nearNodeUrl, futureBlock, txn) {
  const resp = await fetch(nearNodeUrl, {
    method: 'post',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      jsonrpc: '2.0',
      id: 'dontcare',
      method: 'light_client_proof',
      params: {
        type: 'transaction',
        transaction_hash: txn.hash,
        receiver_id: txn.receiver_id,
        sender_id: txn.signer_id,
        light_client_head: futureBlock.header.hash
      }
    })
  })
  const data = await resp.json()
  return data.result
}

async function getReceiptProof (nearNodeUrl, futureBlock, receipt) {
  const resp = await fetch(nearNodeUrl, {
    method: 'post',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      jsonrpc: '2.0',
      id: 'dontcare',
      method: 'light_client_proof',
      params: {
        type: 'receipt',
        receipt_id: receipt.receipt_id,
        receiver_id: receipt.receiver_id,
        light_client_head: futureBlock.header.hash
      }
    })
  })
  const data = await resp.json()
  return data.result
}

async function getNextLightClientBlock (nearNodeUrl, blockHash) {
  const resp = await fetch(nearNodeUrl, {
    method: 'post',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      jsonrpc: '2.0',
      id: 'dontcare',
      method: 'next_light_client_block',
      params: [blockHash]
    })
  })
  const data = await resp.json()
  return data.result
}

class NearDump {
  static async execute (kindOfData, { path, numBlocks, nearNodeUrl }) {
    if (kindOfData !== 'headers' && kindOfData !== 'proofs') {
      console.log(
        'Usage: node index.js near-dump headers\n       node index.js near-dump proofs'
      )
      process.exit(2)
    }

    if (!numBlocks) {
      numBlocks = 100
    }
    const latestBlock = await getLatestBlock(nearNodeUrl)

    if (kindOfData === 'headers') {
      await NearDump.dumpHeaders(nearNodeUrl, path, latestBlock, numBlocks)
    } else if (kindOfData === 'proofs') {
      await NearDump.dumpProofs(nearNodeUrl, path, latestBlock, numBlocks)
    }
  }

  static async dumpHeaders (nearNodeUrl, path, latestBlock, numBlocks) {
    console.log(
      `Downloading ${numBlocks} light client blocks start from ${latestBlock.header.height}`
    )

    let newLatestBlock
    while (numBlocks > 0) {
      newLatestBlock = await getLatestBlock(nearNodeUrl)
      if (newLatestBlock.header.height > latestBlock.header.height) {
        console.log(`Got new block at height ${newLatestBlock.header.height}`)
        let block
        do {
          block = await getNextLightClientBlock(
            nearNodeUrl,
            newLatestBlock.header.hash
          )
        } while (!block)
        console.log(
          `Got new light client block at height ${block.inner_lite.height}`
        )
        await NearDump.saveBlock(block.inner_lite.height, block, path)
        latestBlock = newLatestBlock
        numBlocks--
      } else {
        continue
      }
    }
  }

  static async dumpProofs (nearNodeUrl, path, latestBlock, numBlocks) {
    console.log(
      `Downloading ${numBlocks} light client proofs start from ${latestBlock.header.height}`
    )

    let newLatestBlock
    while (numBlocks > 0) {
      newLatestBlock = await getLatestBlock(nearNodeUrl)
      if (newLatestBlock.header.height > latestBlock.header.height) {
        console.log(`Got new block at height ${newLatestBlock.header.height}`)
        const chunk = await getBlockChunk(nearNodeUrl, latestBlock)
        console.log(
          `There are ${chunk.transactions.length} txns in block ${latestBlock.header.height}'s chunk`
        )
        console.log(
          `There are ${chunk.receipts.length} receipts in block  ${latestBlock.header.height}'s chunk`
        )
        for (const i in chunk.transactions) {
          const proof = await getTxProof(nearNodeUrl, newLatestBlock, chunk.transactions[i])
          await NearDump.saveProof(latestBlock.header.height, 'txn', i, proof, path)
        }
        for (const i in chunk.receipts) {
          const proof = await getReceiptProof(
            nearNodeUrl,
            newLatestBlock,
            chunk.receipts[i]
          )
          await NearDump.saveProof(
            latestBlock.header.height,
            'receipt',
            i,
            proof,
            path
          )
        }
        latestBlock = newLatestBlock
        numBlocks--
      } else {
        continue
      }
    }
  }

  static async saveBlock (i, block, path) {
    const file = Path.join(path, `${i}.json`)
    await fs.writeFile(file, JSON.stringify(block))
  }

  static async saveProof (blockI, type, i, proof, path) {
    const file = Path.join(path, `${blockI}_${type}_${i}.json`)
    await fs.writeFile(file, JSON.stringify(proof))
    console.log('Saved ' + file)
  }
}

exports.NearDump = NearDump
