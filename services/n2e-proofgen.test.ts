import { strict as assert } from 'assert'
import { Server } from 'http'
import { format, promisify } from 'util'

import * as FakeTimers from '@sinonjs/fake-timers'
import { AbstractLevelDOWN, ErrorCallback } from 'abstract-leveldown'
import { program } from 'commander'
import encode from 'encoding-down'
import * as fetchModule from 'node-fetch'
import * as rocksdbModule from 'rocksdb'

/* eslint-disable @typescript-eslint/no-empty-function, @typescript-eslint/require-await */

// Need to make 'memdown' use process.nextTick instead of setImmediate to make fake timers work.
declare let setImmediate: (f: () => any, ...args: any[]) => any
const origSetImmediate = setImmediate
setImmediate = process.nextTick // eslint-disable-line @typescript-eslint/unbound-method
import memdown from 'memdown' // eslint-disable-line import/first
setImmediate = origSetImmediate

// Stub console.error and process.exit for proper error reporting.

let errors: string[]
console.error = (...args: [unknown, ...unknown[]]) => {
  errors.push(format(...args))
}

const origExit = process.exit // eslint-disable-line @typescript-eslint/unbound-method
process.exit = () => {
  throw new Error('An error occurred')
}
// Restore process.exit so that mocha can use it.
after(() => { process.exit = origExit })

// Stub 'http' module

let server: Server
Server.prototype.listen = function () {
  server = this
  process.nextTick(() => { this.emit('listening') })
  return this
}

// Stub 'commander' module.

let done: () => void
const origParseAsync = program.parseAsync // eslint-disable-line @typescript-eslint/unbound-method
program.parseAsync = async function (this: typeof program) {
  // For some reason, 'commander' doesn't work if parseAsync is called multiple times, but this should fix it.
  this._actionResults = []
  try {
    await origParseAsync.call(this)
  } catch (e) {
    if (!errors.length || (e as Error).message !== 'An error occurred') {
      errors.push((e as Error).message)
    }
  }
  done()
  return this
}

/* eslint-disable @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-member-access */

// Stub 'node-fetch' module

interface Block {
  hash: string
  prev_hash: string
  changes: Array<{
    cause: {
      receipt_hash: string
    }
    change: {
      key_base64: string
      value_base64: string
    }
  }>
}

interface Receipt {
  predecessor_id: string
  actions: Array<{
    type: 'call'
    method: string
    args_base64: string
  }>
}

let blocks: Map<number, Block>, finalHeight: number, receipts: Map<string, Receipt>, stopFetch: boolean
beforeEach(() => {
  blocks = new Map<number, Block>()
  finalHeight = 0
  receipts = new Map<string, Receipt>()
  stopFetch = false
})

;(fetchModule as any).default = (_url: string, { body }: { body: string }) => {
  if (stopFetch) {
    return Promise.reject(new Error('shutdown'))
  }
  const { method, params } = JSON.parse(body) as { method: string, params: any }
  let result: unknown, error: unknown
  switch (method) {
    case 'block':
      {
        const height = 'block_id' in params ? params.block_id as number : finalHeight
        const block = blocks.get(height)
        if (block) {
          result = { header: { height, hash: block.hash, prev_hash: block.prev_hash } }
        } else {
          // XXX handle missing block
          error = { code: -32000, message: 'No such block' }
        }
      }
      break
    case 'EXPERIMENTAL_changes':
      {
        const block = blocks.get(params.block_id as number)
        if (block) {
          result = { changes: block.changes }
        } else {
          // XXX handle missing block
          error = { code: -32000, message: 'No such block' }
        }
      }
      break
    case 'UNIMPLEMENTED_getreceipt':
      {
        const receipt = receipts.get(params.hash as string)
        if (receipt) {
          result = receipt
        } else {
          // XXX handle missing receipt
          error = { code: -32000, message: 'No such receipt' }
        }
      }
      break
    default:
      error = { code: -32601, message: 'Method not found' }
  }
  return Promise.resolve({
    ok: true,
    json() { return Promise.resolve({ result, error }) }
  })
}

// Stub 'rocksdb' module

type DB = AbstractLevelDOWN<string | Buffer, string | Buffer> & { db: { _store: { forEach: (visitor: (k: Buffer, v: Buffer) => void) => void } } }
let db: DB
// eslint-disable-next-line @typescript-eslint/unbound-method
const dbEncoding = { encode: Buffer.from, decode: Buffer.from, buffer: true, type: 'copy' }
beforeEach(() => {
  db = encode(memdown(), { keyEncoding: dbEncoding, valueEncoding: dbEncoding }) as DB
})
;(rocksdbModule as any).default = () => db
;(rocksdbModule as any).destroy = (_path: string, cb: ErrorCallback) => { process.nextTick(cb, null) }

/* eslint-enable @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-member-access */

// Fake timers

const flushMicrotasks = promisify(setImmediate)
const clock = FakeTimers.install()

// Tests

async function call(...args: string[]): Promise<void> {
  process.argv = ['node', 'index.js', ...args]
  // eslint-disable-next-line @typescript-eslint/no-dynamic-delete
  delete require.cache[require.resolve('./n2e-proofgen')]
  errors = []
  require('./n2e-proofgen')
  await new Promise<void>(resolve => { done = resolve })
  if (errors.length && errors[errors.length - 1] === 'error: shutdown') {
    errors.pop()
  }
  if (errors.length) {
    const message = errors.join('\n')
    errors.length = 0
    throw new Error(message)
  }
}

function dumpDb(): void {
  console.log('Database contents:')
  db.db._store.forEach((k: Buffer, v: Buffer) => {
    console.log(`${k.toString('hex')} -> ${v.toString('hex')}`)
  })
}

function assertProof(sender: string, messageBase64: string, uid: number, proofBase64: string): Promise<void> {
  const input = Buffer.from(JSON.stringify({
    jsonrpc: '2.0',
    method: 'getproof',
    params: { sender, message_base64: messageBase64 },
    id: 0
  }))

  let done: () => void, fail: (e: unknown) => void
  server.emit('request',
    Object.assign((async function* req() { yield input })(), { method: 'POST', url: '/' }),
    {
      writeHead(code: number) {
        if (code !== 200) {
          fail(new Error(`invalid response code: ${code}`))
        }
        return this // eslint-disable-line @typescript-eslint/no-unsafe-return
      },
      headersSent: false,
      destroy() { },
      end(data: Buffer) {
        const { result, error } = JSON.parse(data.toString()) as { result?: { uid: string, proof_base64: string }, error?: { message: string } }
        if (error) {
          fail(new Error(`RPC error: ${error.message}`))
          return
        }
        const { uid: gotUid, proof_base64 } = result!
        try {
          assert.equal(gotUid, String(uid))
          assert.equal(proof_base64, proofBase64)
        } catch (e) {
          fail(e)
          return
        }
        done()
      }
    }
  )

  return new Promise((resolve, reject) => {
    done = resolve
    fail = reject
  })
}

// TODO: test much more: missing heights, unrelated changes/actions/fields/etc, error handling.
it('works', async function () {
  blocks.set(finalHeight = 100500, {
    hash: '4EbsYN7x4nKviz8KoJseUTBCvN6MNQYxMQXm2WBTnYSp',
    prev_hash: '9U7amJHLhZDrUsjAjLb8mtEHuAb7T9JpDiPboq8GR14T',
    changes: []
  })
  console.log('Before init')
  await call('init', '100500')

  console.log('Before run')
  let pRun = call('run')
  await flushMicrotasks()
  console.log('Before first tick')
  clock.tick(1500)
  await flushMicrotasks()
  blocks.set(finalHeight = 100501, {
    hash: 'EXExPC2cgikeBXnNtAj75bgBaEzcafsVarLaZBUdxKkY',
    prev_hash: '4EbsYN7x4nKviz8KoJseUTBCvN6MNQYxMQXm2WBTnYSp',
    changes: [{
      cause: { receipt_hash: '54L8AzrkLzrsVdoorXNpTdZSsWr5ypRXMVb7sssS9Vvt' },
      change: { key_base64: '', value_base64: 'YYPSXGg8dk659jeKA8CIpuMtLLpud+KcKu2oSrL8s7c=' }
    }]
  })
  receipts.set('54L8AzrkLzrsVdoorXNpTdZSsWr5ypRXMVb7sssS9Vvt', {
    predecessor_id: 'ebrjsdxgvoi',
    actions: [{
      type: 'call',
      method: 'send',
      args_base64: 'CAAAABk3D0rmXA53'
    }]
  })
  console.log('Before second tick')
  clock.tick(2000)
  await flushMicrotasks()
  blocks.set(100502, {
    hash: 'GT14FYXYeRWgDgoZdgej9AzEwQjsMypuwCfvMTjowBmg',
    prev_hash: 'EXExPC2cgikeBXnNtAj75bgBaEzcafsVarLaZBUdxKkY',
    changes: []
  })
  blocks.set(finalHeight = 100503, {
    hash: '7AMvT7ThaGP4UY9L2jd89ee6P9xu9nT43NU6Ltch58qQ',
    prev_hash: 'GT14FYXYeRWgDgoZdgej9AzEwQjsMypuwCfvMTjowBmg',
    changes: [{
      cause: { receipt_hash: 'BEkrbMTg5eGZLyaTKyZ4tuFaZ4g83QRjeVzwLW2tV5DS' },
      change: { key_base64: '', value_base64: 'K0seaIqvpGtmvgA28NzG7uCJIXRr98oeZRgC+07FzYI=' }
    }]
  })
  receipts.set('BEkrbMTg5eGZLyaTKyZ4tuFaZ4g83QRjeVzwLW2tV5DS', {
    predecessor_id: 'nxafvgnegntc',
    actions: [{
      type: 'call',
      method: 'foo',
      args_base64: 'F9BJYRPUe2I8RI4nFZmxsg2o8JE='
    }, {
      type: 'call',
      method: 'send',
      args_base64: '5gAAABX75HnY0315snlGq6/apBWGcrmh/XgDihxzgQCzlIL6ed49t6yfCHzslQmFysVZhnfNWEAv2sIzuECDhB5Yvvj8lhcWwsdXdrlkQwJYv8lS6uC3gD1Z6sCSfW2bhVwoGMSihLWkIWLK6WZQzar1C3HX3Zog747kml0VLUgY3pmNIlXIjOyMixhXSs96Aj56SqZcHLEy8QmZUoMKUznJfLj0PQp0lUnyx44hiKrSzD8gV4LwOyrgeezxN/22YM7bpMXNUWaI6OvGF2L6pMsDnSBUvISoP4IV5a8Fm4sIEHVppNj7ISBc'
    }]
  })
  console.log('Before third tick')
  clock.tick(2000)
  await flushMicrotasks()
  stopFetch = true
  console.log('Flushing')
  while (clock.countTimers()) {
    clock.next()
    await flushMicrotasks()
  }
  console.log('Wrapping up')
  await pRun

  dumpDb()
  await assertProof('ebrjsdxgvoi', 'GTcPSuZcDnc=', 0, 'AAAAAAAAAAKjBRDHEZy4r2oRG1iJJxOTk6Qz4CaAT1rfloFI8E62Dg==')
  await assertProof('nxafvgnegntc', 'FfvkedjTfXmyeUarr9qkFYZyuaH9eAOKHHOBALOUgvp53j23rJ8IfOyVCYXKxVmGd81YQC/awjO4QIOEHli++PyWFxbCx1d2uWRDAli/yVLq4LeAPVnqwJJ9bZuFXCgYxKKEtaQhYsrpZlDNqvULcdfdmiDvjuSaXRUtSBjemY0iVciM7IyLGFdKz3oCPnpKplwcsTLxCZlSgwpTOcl8uPQ9CnSVSfLHjiGIqtLMPyBXgvA7KuB57PE3/bZgztukxc1RZojo68YXYvqkywOdIFS8hKg/ghXlrwWbiwgQdWmk2PshIFw=', 1, 'AAAAAAAAAAIdwEavumsKpTCUL5r7rOberR04lAEWm3oDACvo7/mDuw==')

  stopFetch = false
  blocks.set(finalHeight = 100504, {
    hash: 'EHh4vsGqAJZYNCRabt2YP719352LKfbpt1yoDpGTc3t9',
    prev_hash: '7AMvT7ThaGP4UY9L2jd89ee6P9xu9nT43NU6Ltch58qQ',
    changes: [{
      cause: { receipt_hash: 'BMTWvnxn3KD4dT4reyqN67qBY8jyMKLc1EBQj5KZswNG' },
      change: { key_base64: '', value_base64: 'WMjn/kyPXsTWc0hmnPlceVjTlC6NJLfPVbGFJ1C/Zgo=' }
    }, {
      cause: { receipt_hash: 'EnhdKddRk1fsnZiiYAzxaTtYb8dZGyzWQSt8zVSTHtFa' },
      change: { key_base64: '', value_base64: 'r9f8tFiS68RmFun3sy002uHhMPCMqNnxWtxbUFV5NeU=' }
    }]
  })
  receipts.set('BMTWvnxn3KD4dT4reyqN67qBY8jyMKLc1EBQj5KZswNG', {
    predecessor_id: 'nwepupqo',
    actions: [{
      type: 'call',
      method: 'send',
      args_base64: 'AwAAABnZTw=='
    }]
  })
  receipts.set('EnhdKddRk1fsnZiiYAzxaTtYb8dZGyzWQSt8zVSTHtFa', {
    predecessor_id: 'lthspk',
    actions: [{
      type: 'call',
      method: 'send',
      args_base64: 'BgAAAO8hX9cbvA=='
    }]
  })
  console.log('Before second run')
  pRun = call('run')
  console.log('Before first tick')
  clock.tick(1500)
  await flushMicrotasks()
  stopFetch = true
  console.log('Flushing')
  while (clock.countTimers()) {
    clock.next()
    await flushMicrotasks()
  }
  console.log('Wrapping up')
  await pRun

  dumpDb()
  await assertProof('ebrjsdxgvoi', 'GTcPSuZcDnc=', 0, 'AAAAAAAAAASjBRDHEZy4r2oRG1iJJxOTk6Qz4CaAT1rfloFI8E62DsFtDmt1lKIee9FA+SoFhAvvXszTgBZsX9FYnfcamw3E')
  await assertProof('nxafvgnegntc', 'FfvkedjTfXmyeUarr9qkFYZyuaH9eAOKHHOBALOUgvp53j23rJ8IfOyVCYXKxVmGd81YQC/awjO4QIOEHli++PyWFxbCx1d2uWRDAli/yVLq4LeAPVnqwJJ9bZuFXCgYxKKEtaQhYsrpZlDNqvULcdfdmiDvjuSaXRUtSBjemY0iVciM7IyLGFdKz3oCPnpKplwcsTLxCZlSgwpTOcl8uPQ9CnSVSfLHjiGIqtLMPyBXgvA7KuB57PE3/bZgztukxc1RZojo68YXYvqkywOdIFS8hKg/ghXlrwWbiwgQdWmk2PshIFw=', 1, 'AAAAAAAAAAQdwEavumsKpTCUL5r7rOberR04lAEWm3oDACvo7/mDu8FtDmt1lKIee9FA+SoFhAvvXszTgBZsX9FYnfcamw3E')
  await assertProof('nwepupqo', 'GdlP', 2, 'AAAAAAAAAARyhT/YMEHwzmOTvQhznntjvISbj7vikZPl4t/ODcD5aRIRvo4izuO/UjZmiIfEfeVfI4QtV0lFeIZcqrw9LGpm')
  await assertProof('lthspk', '7yFf1xu8', 3, 'AAAAAAAAAASFrcM0Hav5fU6i00xR4uXNXOHuiWTWX2Df1v/T51UQExIRvo4izuO/UjZmiIfEfeVfI4QtV0lFeIZcqrw9LGpm')
})
