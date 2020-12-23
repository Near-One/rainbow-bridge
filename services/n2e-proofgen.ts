import { once } from 'events'
import { IncomingMessage, Server, ServerResponse } from 'http'

/* eslint-disable import/first */
import { AbstractLevelDOWN, ErrorCallback, ErrorValueCallback } from 'abstract-leveldown'
import * as bs58 from 'bs58'
import { program } from 'commander'
import Denque = require('denque')
import { Keccak } from 'keccak'
// eslint-disable-next-line @typescript-eslint/no-var-requires
const createKeccakHash = require('keccak') as (algorithm: 'keccak256') => Keccak
import fetch from 'node-fetch'
import rocksdb, { destroy as destroyDb } from 'rocksdb'
/* eslint-enable import/first */

// Definitions in @types/keccak and @types/rocksdb are not correct, need to fix.
declare module 'rocksdb' {
  export function destroy(location: string, cb: (err: Error | undefined) => void): void
}

type DB = AbstractLevelDOWN<Buffer, Buffer>

interface RpcReqMsg<T> {
  jsonrpc: '2.0'
  method: string
  params: T
  id?: string | number | null
}

interface RpcResMsg<T> {
  jsonrpc: '2.0'
  result?: T
  error?: RpcError
  id: string | number | null
}

interface RpcError {
  code: number
  message: string
  data?: unknown
}

interface RpcSchema {
  block: {
    request: {
      block_id: number
    } | {
      finality: 'final'
    }
    response: {
      header: {
        height: number
        hash: string
        prev_hash: string
      }
    }
  }
  EXPERIMENTAL_changes: {
    request: {
      changes_type: 'data_changes'
      account_ids: [string]
      key_prefix_base64: string
      block_id: number
    }
    response: {
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
  }
  UNIMPLEMENTED_getreceipt: {
    request: {
      hash: string
    }
    response: {
      predecessor_id: string
      actions: Array<{
        type: 'call'
        method: string
        args_base64: string
      }>
    }
  }
}

type RpcMethods = keyof RpcSchema
type RpcReq<M extends RpcMethods> = RpcSchema[M]['request']
type RpcRes<M extends RpcMethods> = RpcSchema[M]['response']

// TODO keep alive connections
interface Config {
  'db-path': string
  'rpc-url': string
  'account-id': string
  'poll-delay': number
  'pipeline-depth': number
  'server-addr': { host: string, port: number }
}
const {
  'db-path': dbPath,
  'rpc-url': rpcURL,
  'account-id': accountId,
  'poll-delay': pollDelay,
  'pipeline-depth': pipelineDepth,
  'server-addr': serverAddr
// eslint-disable-next-line @typescript-eslint/no-var-requires
} = require('./config.json') as Config

function sleep(t: number): Promise<void> {
  console.log(`Sleep ${t}`)
  return new Promise(resolve => { setTimeout(resolve, t) })
}

// TODO retry on failure
async function rpcRequest<M extends RpcMethods>(method: M, params: RpcReq<M>): Promise<RpcRes<M>> {
  console.log(`Begin RPC ${method} ${JSON.stringify(params)}`)
  const res = await fetch(rpcURL, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ jsonrpc: '2.0', method, params, id: 0 })
  })
  if (!res.ok) {
    throw new Error(`RPC returned error status ${res.status}`)
  }
  const { result, error } = await res.json() as RpcResMsg<RpcRes<M>>
  if (error !== undefined) {
    console.log(`Error RPC ${method} ${JSON.stringify(params)} ${JSON.stringify(error)}`)
    throw new Error(`RPC returned error ${error.code}: ${error.message}`)
  }
  console.log(`Success RPC ${method} ${JSON.stringify(params)} ${JSON.stringify(result)}`)
  return result!
}

function rpcAssert(cond: boolean): asserts cond {
  if (!cond) {
    throw new Error('RPC returned invalid data')
  }
}

function rpcDecodeHash(v: unknown): Buffer {
  rpcAssert(typeof v === 'string')
  const res = bs58.decode(v)
  rpcAssert(res.length === 32)
  return res
}

function isObject(v: unknown): boolean {
  return typeof v === 'object' && v !== null && !Array.isArray(v)
}

function isValidAccountId(v: unknown): boolean {
  return typeof v === 'string' && v.length >= 2 && v.length <= 64 && /^[0-9a-z]+(?:[-._][0-9a-z]+)*$/.test(v)
}

function tryDecodeBase64Strict(v: unknown): Buffer | null {
  let res
  return typeof v !== 'string' || (res = Buffer.from(v, 'base64')).toString('base64') !== v ? null : res
}

function highestOneBit(n: bigint): number {
  let res = 0
  for (let i = 32; i > 0; i >>= 1) {
    if (n >= 1n << BigInt(i)) {
      n >>= BigInt(i)
      res += i
    }
  }
  return res
}

function trailingOnes(n: bigint): number {
  let res = 0
  for (let i = 32; i > 0; i >>= 1) {
    if ((~n & ((1n << BigInt(i)) - 1n)) === 0n) {
      n >>= BigInt(i)
      res += i
    }
  }
  return res
}

type PC<A extends unknown[], R> = (...args: [...A, ErrorValueCallback<R>]) => any
type PMC<T, M extends keyof T, A extends unknown[], R> = {
  [Method in M]: (this: T, ...args: [...A, ErrorValueCallback<R>]) => any
}
type PR<F> =
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  F extends (...args: [...infer _A, infer C]) => any ?
    // eslint-disable-next-line @typescript-eslint/no-invalid-void-type
    C extends ErrorCallback ? void :
      C extends (err: Error | undefined, res: infer R, ...rest: any) => void ? R : unknown : unknown

function promisifyCall<F extends PC<A, R>, A extends unknown[], R = PR<F>>(fn: F, ...args: A): Promise<R> {
  return new Promise((resolve, reject) => {
    fn(...args, (err: Error | undefined, res: R) => {
      if (err) {
        reject(err)
      } else {
        resolve(res)
      }
    })
  })
}

function promisifyMethodCall<T extends PMC<T, M, A, R>, M extends keyof T,
  A extends unknown[], R = PR<T[M]>>(receiver: T, method: M, ...args: A): Promise<R> {
  return new Promise((resolve, reject) => {
    receiver[method](...args, (err: Error | undefined, res: R) => {
      if (err) {
        reject(err)
      } else {
        resolve(res)
      }
    })
  })
}

// eslint-disable-next-line @typescript-eslint/no-empty-function
function nop(): void { }
function iur<T, U extends Promise<T>>(p: U): U {
  p.catch(nop)
  return p
}

function command(cmd: string, action: (...args: string[]) => Promise<void>): typeof program {
  return program.command(cmd)
    .action(async (...args) => {
      if (Array.isArray(args[args.length - 1])) {
        console.error('error: too many arguments')
        process.exit(1)
      }
      try {
        return await action(...args)
      } catch (e) {
        console.error(`error: ${(e as Error).message}`)
        process.exit(1)
      }
    })
}

async function withDb<T>(opts: any, fn: (db: DB) => Promise<T>): Promise<T> {
  const db = rocksdb(dbPath) as DB
  await promisifyMethodCall(db, 'open', opts)
  try {
    return await fn(db)
  } finally {
    await promisifyMethodCall(db, 'close')
  }
}

const MAX_N = 1n << 63n
const EMPTY = Buffer.alloc(0)

command('init <height>', async (height: any) => {
  if (!/^[0-9]+$/.test(height) || (height = Number(height)) > Number.MAX_SAFE_INTEGER) {
    throw new Error('Invalid height')
  }
  try {
    await withDb({ errorIfExists: true }, async db => {
      const { header: { hash } } = await rpcRequest('block', { block_id: Number(height) })
      const state = Buffer.alloc(48)
      state.writeBigUInt64LE(BigInt(height))
      rpcDecodeHash(hash).copy(state, 8)
      await promisifyMethodCall(db, 'put', Buffer.from([0]), state)
    })
  } catch (e) {
    await promisifyCall(destroyDb, dbPath)
    throw e
  }
})

// TODO error handling
// TODO logging
// TODO disable syncing, disable server
// TODO api
command('run', () => withDb({ createIfMissing: false }, async db => {
  console.log('Begin run')
  let state = await promisifyMethodCall(db, 'get', Buffer.from([0]))
  let procHeight: number, n: bigint, procN: bigint
  if (state.length < 48 ||
    (procHeight = Number(state.readBigUInt64LE()), procHeight > Number.MAX_SAFE_INTEGER) ||
    (n = state.readBigUInt64BE(40), n > MAX_N || state.length !== 48 + 32 * highestOneBit(n + 1n))) {
    throw new Error('invalid state')
  }
  procN = n

  console.log('Begin server setup')
  const server = new Server()
  // eslint-disable-next-line @typescript-eslint/no-misused-promises
  server.on('request', handleRequest)
  server.listen(serverAddr)
  await once(server, 'listening')

  let error: unknown
  const nstate = Buffer.allocUnsafeSlow(state.length + 32)
  state.copy(nstate)
  state = nstate
  const queue = new Denque<{ block: RpcRes<'block'> | Promise<RpcRes<'block'>>, receipts: Promise<Change[] | null> }>()
  type Signal = (() => void) | undefined
  let emptySignal: Signal, fullSignal: Signal
  try {
    void fetchBlocks(procHeight + 1)
    let curHash = bs58.encode(state.slice(8, 40))

    console.log('Begin process loop')
    while (true) {
      if (error === undefined && queue.isEmpty()) {
        await new Promise<void>(resolve => { emptySignal = resolve })
      }
      if (error !== undefined) {
        throw error
      }
      const { block, receipts } = queue.shift()!
      const { header: { height, hash, prev_hash } } = await block // XXX handle missing height
      if (prev_hash !== curHash) {
        throw new Error('wrong previous block')
      }
      curHash = hash
      const batch = db.batch()
      for (const { changes, data } of (await receipts)!) { // XXX handle missing height
        let i = 0
        // XXX get receipt
        const { predecessor_id, actions } = await data
        rpcAssert(isValidAccountId(predecessor_id))
        for (const { type, method, args_base64 } of actions) {
          if (type === 'call' && method === 'send') {
            rpcAssert(typeof args_base64 === 'string')
            const args = Buffer.from(args_base64, 'base64')
            if (args.length < 4 || args.length !== args.readUInt32LE() + 4) {
              throw new Error('invalid argument encoding')
            }
            if (i >= changes.length) {
              throw new Error('number of actions doesn\'t match the number of changes')
            }
            if (n >= MAX_N) {
              throw new Error('message height overflow')
            }
            const msg = createKeccakHash('keccak256')
              .update(Buffer.from([predecessor_id.length]))
              .update(predecessor_id)
              .update(args.slice(4))
              .digest()
            console.log(`Processing state update. N: ${n} -> ${n + 1n}, msg: ${msg.toString('hex')}`)
            const k1 = Buffer.allocUnsafe(9)
            k1[0] = 1
            k1.writeBigUInt64LE(n << 1n, 1)
            console.log(`Put: ${k1.toString('hex')} ${msg.toString('hex')}`)
            batch.put(k1, msg)
            const k2 = Buffer.allocUnsafe(41)
            k2[0] = 2
            msg.copy(k2, 1)
            k2.writeBigUInt64BE(n, 33)
            console.log(`Put: ${k2.toString('hex')} ${EMPTY.toString('hex')}`)
            batch.put(k2, EMPTY)
            n++
            state.writeBigUInt64BE(n, 40)
            msg.copy(state, state.length - 32)
            if ((n & (n + 1n)) === 0n) {
              const nstate = Buffer.allocUnsafeSlow(state.length + 32)
              state.copy(nstate)
              state = nstate
            } else {
              const t = trailingOnes(n)
              const p = state.length - (t << 5)
              const h = createKeccakHash('keccak256').update(state.slice(p - 64, p)).digest()
              k1.writeBigUInt64LE((n << 1n) - (2n << BigInt(t)) - 1n, 1)
              console.log(`Put: ${k1.toString('hex')} ${h.toString('hex')}`)
              batch.put(k1, h)
              h.copy(state, p - 64)
              state.copy(state, p - 32, p)
            }
            console.log(`New state: ${state.slice(40).toString('hex')}`)
            let cur = state.slice(48, 80)
            for (let p = 112; p < state.length; p += 32) {
              cur = createKeccakHash('keccak256')
                .update(cur)
                .update(state.slice(p - 32, p))
                .digest()
            }
            const root = createKeccakHash('keccak256')
              .update(state.slice(40, 48))
              .update(cur)
              .digest()
            console.log(`Computed pre-root hash: ${cur.toString('hex')}, root hash: ${root.toString('hex')}, found: ${changes[i].toString('hex')}`)
            if (!root.equals(changes[i])) {
              throw new Error('root hash doesn\'t match')
            }
            i++
          }
        }
        if (i !== changes.length) {
          throw new Error('number of actions doesn\'t match the number of changes')
        }
      }
      state.writeBigUInt64LE(BigInt(height))
      rpcDecodeHash(hash).copy(state, 8)
      // TODO checks
      batch.put(Buffer.from([0]), state.slice(0, state.length - 32))
      await promisifyMethodCall(batch, 'write')
      procHeight = height
      if (fullSignal) {
        fullSignal()
        fullSignal = undefined
      }
      procN = n
    }
  } catch (e) {
    // TODO abort RPC
    if (error === undefined) {
      error = e
      if (fullSignal) {
        fullSignal()
      }
    }
    throw error
  }

  async function fetchBlocks(fetchHeight: number): Promise<void> {
    try {
      let lastTime
      console.log('Begin fetch loop')
      while (true) {
        let time = Date.now()
        console.log(`Time ${time}, last time ${String(lastTime)}`)
        if (lastTime !== undefined && time - lastTime < pollDelay) {
          // TODO abort
          await sleep(pollDelay - (time - lastTime))
          time = Date.now()
        }
        lastTime = time
        const finalBlock = await rpcRequest('block', { finality: 'final' })
        const finalHeight = finalBlock.header.height
        if (!(finalHeight <= Number.MAX_SAFE_INTEGER)) {
          throw new Error('block height overflow')
        }
        for (; fetchHeight <= finalHeight; fetchHeight++) {
          if (error === undefined && fetchHeight - procHeight > pipelineDepth) {
            await new Promise<void>(resolve => { fullSignal = resolve })
          }
          if (error !== undefined) {
            return
          }
          const block = fetchHeight < finalHeight ? iur(rpcRequest('block', { block_id: fetchHeight })) : finalBlock
          queue.push({ block, receipts: iur(getChanges(fetchHeight)) })
          if (emptySignal) {
            emptySignal()
            emptySignal = undefined
          }
        }
      }
    } catch (e) {
      if (error === undefined) {
        error = e
        if (emptySignal) {
          emptySignal()
        }
      }
    }
  }

  interface Change {
    changes: Buffer[]
    data: Promise<RpcRes<'UNIMPLEMENTED_getreceipt'>>
  }

  async function getChanges(height: number): Promise<Change[] | null> {
    let changes
    try {
      changes = await rpcRequest('EXPERIMENTAL_changes', {
        changes_type: 'data_changes',
        account_ids: [accountId],
        key_prefix_base64: '',
        block_id: height
      })
    } catch (e) {
      // XXX handle missing height
      return null
    }
    let lastHash, lastChanges: Buffer[]
    const receipts = []
    for (const {
      cause: { receipt_hash },
      change: { key_base64, value_base64 }
    } of changes.changes) {
      if (key_base64 === '') {
        if (receipt_hash !== lastHash) {
          receipts.push({
            changes: lastChanges = [],
            data: iur(rpcRequest(
              // XXX get receipt
              'UNIMPLEMENTED_getreceipt',
              { hash: lastHash = receipt_hash }
            ))
          })
        }
        lastChanges!.push(Buffer.from(value_base64, 'base64'))
      }
    }
    return receipts
  }

  interface ReqObj {
    sender: string
    message_base64: string
  }
  type ReqArr = [string, string]
  type Req = ReqObj | ReqArr

  interface Res {
    uid: string
    proof_base64: string
  }

  async function handleRequest(req: IncomingMessage, res: ServerResponse): Promise<void> {
    try {
      if (req.url !== '/') {
        res.writeHead(404)
      } else if (req.method !== 'POST') {
        res.writeHead(405)
      } else {
        const chunks = []
        for await (const chunk of req) {
          chunks.push(chunk)
        }
        let input: RpcReqMsg<Req>, method, params, id
        const output: RpcResMsg<Res> = { jsonrpc: '2.0' } as any as RpcResMsg<Res>
        output: {
          console.log(`Input: ${Buffer.concat(chunks).toString()}`)
          try {
            input = JSON.parse(new TextDecoder('utf-8', { fatal: true, ignoreBOM: true }).decode(Buffer.concat(chunks))) as RpcReqMsg<Req>
          } catch (e) {
            console.log(`Error: ${String(e)}`)
            output.error = { code: -32700, message: 'Parse error' }
            break output
          }
          if (!isObject(input) ||
            input.jsonrpc !== '2.0' ||
            typeof (method = input.method) !== 'string' ||
            (!isObject(params = input.params) && !Array.isArray(params)) ||
            ((id = input.id) != null && typeof id !== 'string' && typeof id !== 'number')) {
            output.error = { code: -32600, message: 'Invalid Request' }
            break output
          }
          output.id = id!
          if (method !== 'getproof') {
            output.error = { code: -32601, message: 'Method not found' }
            break output
          }
          let sender: string | undefined, message_base64, message
          if (isObject(params)) {
            ({ sender, message_base64 } = params as ReqObj)
          } else if ((params as ReqArr).length === 2) {
            [sender, message_base64] = params as ReqArr
          }
          if (!isValidAccountId(sender) ||
            (message = tryDecodeBase64Strict(message_base64)) === null) {
            output.error = { code: -32602, message: 'Invalid params' }
            break output
          }
          const msg = createKeccakHash('keccak256')
            .update(Buffer.from([sender!.length]))
            .update(sender!)
            .update(message)
            .digest()
          const n = procN
          const gte = Buffer.allocUnsafe(33)
          gte[0] = 2
          msg.copy(gte, 1)
          const lt = Buffer.allocUnsafe(41)
          gte.copy(lt)
          lt.writeBigUInt64BE(n, 33)
          const it = db.iterator({ gte, lt, reverse: true })
          let ki
          try {
            ki = await promisifyMethodCall(it, 'next')
          } finally {
            await promisifyMethodCall(it, 'end')
          }
          if (ki === undefined) {
            output.error = { code: 1, message: 'Event not found' }
            break output
          }
          const uid = ki.readBigUInt64BE(33)
          const q = []
          const kp = Buffer.allocUnsafe(9)
          kp[0] = 1
          const d2 = highestOneBit((((n + 1n) ^ uid) >> 1n) & ((n + 1n) | ~uid))
          const d0 = d2 + +!!((n + 1n) & (1n << BigInt(d2)))
          console.log(`N = ${n}, I = ${uid}, d2 = ${d2}, d0 = ${d0}`)
          let ci = uid
          for (let i = 0; i < d0; i++) {
            const bit = 1n << BigInt(i)
            kp.writeBigUInt64LE(((ci ^= bit) << 1n) + bit - 1n, 1)
            console.log(`Get: ${(ci << 1n) + bit - 1n} ${kp.toString('hex')}`)
            q.push(promisifyMethodCall(db, 'get', kp))
            ci &= ~bit
          }
          let p = -1n
          for (let i = highestOneBit(n + 1n) - 1; i > d2; i--) {
            const s = 1n << BigInt(i + +!!((n + 1n) & (1n << BigInt(i))))
            kp.writeBigUInt64LE(p += s, 1)
            console.log(`Get: ${p} ${kp.toString('hex')}`)
            q.push(promisifyMethodCall(db, 'get', kp))
            p += s
          }
          const dl = q.length
          p += 2n << BigInt(d0)
          for (let i = d2 - 1; i >= 0; i--) {
            const s = 1n << BigInt(i + +!!((n + 1n) & (1n << BigInt(i))))
            kp.writeBigUInt64LE(p += s, 1)
            console.log(`Get: ${p} ${kp.toString('hex')}`)
            q.push(promisifyMethodCall(db, 'get', kp))
            p += s
          }
          const r = await Promise.all(q)
          if (dl > d0 + 1) {
            let cur = r[d0]
            for (let i = d0 + 1; i < dl; i++) {
              cur = createKeccakHash('keccak256')
                .update(cur)
                .update(r[i])
                .digest()
            }
            r.splice(d0, dl - d0, cur)
          }
          const proof = Buffer.allocUnsafe(8 + (r.length << 5))
          proof.writeBigUInt64BE(n)
          for (let i = 0, j = 8; i < r.length; i++, j += 32) {
            r[i].copy(proof, j)
          }
          output.result = { uid: String(uid), proof_base64: proof.toString('base64') }
        }
        const data = Buffer.from(JSON.stringify(output))
        res.writeHead(200, { 'Content-Type': 'application/json; charset=UTF-8', 'Content-Length': data.length }).end(data)
      }
    } catch (e) {
      console.log((e as Error).stack)
      if (!res.headersSent) {
        try {
          res.writeHead(400).end()
        } catch (e) { /* ignore */ }
      }
      res.destroy()
    }
  }
}))

void program.parseAsync()
