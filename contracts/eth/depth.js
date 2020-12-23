// Depth function: for 0 <= i < n, represents n as the sum
// 2^(k-1+b[k-1]) + 2^(k-2+b[k-2]) + ... + 2^(0+b[0])
// for some k and 0 <= b[0], b[1], ..., b[k-1] <= 1
// and finds 0 <= d < k such that i is less than the sum of the first k-d terms but not k-d-1.

'use strict'

const assert = require('assert').strict

function split(n) {
  assert(Number.isInteger(n) && 0 <= n && n < 2 ** 30 - 1)
  let s = 0
  const vs = []
  while (2 * s < n) {
    vs.push(s + 1)
    s += s + 1
  }
  for (let i = vs.length - 1; i >= 0; i--) {
    if (s + vs[i] <= n) {
      s += vs[i]
      vs[i] += vs[i]
    }
  }
  assert(s === n)
  return vs
}

// eslint-disable-next-line no-unused-vars
function depth(n, i) {
  assert(Number.isInteger(n) && Number.isInteger(i) && 0 <= i && i < n && n < 2 ** 30 - 1)
  const vs = split(n)
  let p = vs.length - 1
  let s = 0
  while (i >= s + vs[p]) {
    s += vs[p]
    p--
  }
  return p
}

function highestOneBitIndex(n) {
  for (let i = 0; i < 32; i++) {
    if ((1 << i) > n && n >= 0) {
      return i - 1
    }
  }
  return 31
}

/*
for (let n = 0; n < 10; n++) {
  let ds = []
  for (let i = 0; i < n; i++) {
    let d = depth(n, i)
    let d1 = highestOneBitIndex((((n + 1) ^ i) >> 1) & (~i | (n + 1)))
    if (d != d1) {
      console.log('V1', n, i, d, d1)
    }
    let x = highestOneBitIndex(n - i)
    let y = (1 << x) - 1
    let d2 = x - (n - i <= y + ((n + 1) & y))
    if (d != d2) {
      console.log('V2', n, i, d, d2)
    }
    ds.push(d)
  }
  //console.log(split(n), ds)
}
*/

for (let n = 0; n < 10; n++) {
  const v = []
  for (let i = 0; i <= 2 * n + 10; i++) {
    const d1 = highestOneBitIndex((((n + 1) ^ i) >> 1) & (~i | (n + 1)))
    v.push(d1)
  }
  console.log(v)
}
