# Ethash proof - Command line to calculate ethash (ethereum POW) merkle proof

Ethashproof is a commandline to calculate proof data for an ethash POW, it is used by project `SmartPool` and a decentralized
bridge between Etherum and EOS developed by Kyber Network team.

## Features

1. Calculate merkle root of the ethash dag dataset with given epoch
2. Calculate merkle proof of the pow (dataset elements and their merkle proofs) given the pow submission with given block header
3. Generate dag dataset

## Installation

1. Install go1.11.2 (https://golang.org/doc/install#install)
2. Run export GO111MODULE=on 
3. Run `./build.sh`

## Usage

`ethashproof` comes with 3 tools:
1. `cmd/epoch/epoch` which calculate merkle root for epochs from [0, 512)
2. `cmd/relayer/relayer` which accepts block number to calculate all necessary information in order to prove the block
3. `cmd/cache/cache` which calculate cache merkle tree for an epoch

### The output

When you run `cmd/epoch/epoch`, it will print:
1. Merkle root of the DAG dataset

When you run `cmd/relayer/relayer`, it will print a json containing following information:

1. RLP encoding of the block header
2. Merkle root of the DAG dataset
3. An array of DAG dataset element arrays (that were used in ethash POW). The array is flatten. (Check DAG data element encoding section)
4. An array of all merkle proofs for all of the elements above (check Merkle audit proof encoding section)
5. Length of the proof for a DAG dataset element

*note: the first time you run `relayer` for a blockno corresponding to a new epoch, `relayer` will take several minutes to generate the DAG dataset and calculate everything as well as preparing the cache in order to improve its performance in next runs. It is recommended to run `epoch` for future epoches so it can prepare everything beforehand.*

## Explanations

### Merkle tree in ethashproof

In `ethashproof`, we construct a merkle tree out of ethash DAG dataset in order to get merkle root
of the dataset and get merkle proof for any specific dataset element.
Each DAG dataset is a sequence of many 128 bytes dataset elements, denoted as:
```
e0, e1, e2, ..., en
```

#### Merkle tree explanation

The merkle tree is constructed in the following way:

- Step 1: Calculate hash of each element using `elementhash` function. We would have:
```
h00, h01, h02, h03, ..., h0n
 |    |    |    |   ...   |
e0,  e1,  e2,  e3,  ..., en
```
where `h01` means hash at level 0, element 1. The hash is 32 bytes.

- Step 2: In this step, set the `working level` to 0.
calculate `h10 = hash(h00, h01)`, `h11 = hash(h02, h03)`, ...
if at the end of the level, there is only one element left, we duplicate it and calculate the hash out of them, eg. `h0x = hash(h0x*2, h0x*2)`
```
   h10       h11    ...  h1n/2
  /  \      /  \           /  \
h00, h01, h02, h03, ..., h0n  h0n
 |    |    |    |   ...   |
e0,  e1,  e2,  e3,  ..., en
```

- Step 3: Increase `working level` by 1 and go back to Step 2 until there is only 1 element in the working level. That element is the merkle root.
```
            merkle root
                .
              .....
            ..........
          ...............
        h20         ...  h2n/4
      /     \          /   |
   h10       h11    ...  h1n/2
  /  \      /  \           /  \
h00, h01, h02, h03, ..., h0n  h0n
 |    |    |    |   ...   |
e0,  e1,  e2,  e3,  ..., en
```

#### Hash function
0. Given `sha256(bytes) => bytes` is a function to hash an array of bytes
1. Hash function for data element(`elementhash`)
`elementhash` returns 16 bytes hash of the dataset element.
```
function elementhash(data) => 16bytes {
  h = sha256(conventional(data)) // conventional function is defined in dataset element encoding section
  return last16Bytes(h)
}
```

2. Hash function for 2 sibling nodes (`hash`)
`hash` returns 16 bytes hash of 2 consecutive elements in a working level.
```
function hash(a, b) => 16bytes {
  h = sha256(zeropadded(a), zeropadded(b)) // where zeropadded function prepend 16 bytes of 0 to its param	  h = sha256(a, b) // where a, b are 32bytes
  return last16Bytes(h)
}
```

#### Conventional encoding

To make it easier for ethereum smartcontract to follow the hash calculation, we use a convention to encode DAG dataset element
to use in hash function. The encoding is defined as the following pseudo code:

1. assume the element is `abcd` where a, b, c, d are 32 bytes word
2. `first = concat(reverse(a), reverse(b))` where `reverse` reverses the bytes
3. `second = concat(reverse(c), reverse(d))`
4. conventional encoding of `abcd` is `concat(first, second)`

#### Dataset element encoding

In order to make it easy (and gas saving) for ethereum smart contract (the earliest contract we used to verify the proof) to work with the
dataset element, `ethashproof` outputs a DAG dataset element as an array of 32 bytes word, the word is little endian.

#### Merkle audit proof

Please read more on http://www.certificate-transparency.org/log-proofs-work.

#### Merkle audit proof encoding

For a DAG dataset element, there is a list of hashes (the proof) to prove its existence. In `ethashproof`, we dont include dataset element's hash	
and the merkle root in the proof and format it in the following rules

1. assume the hashes are: `[h0, h1, h2, h3, ..., hn]` where `hi` is 16 bytes.
2. if n is odd, append a 16 bytes number of 0.
3. reorder the hashes to: `[h1, h0, h3, h2, ...]`
4. concatenate 2 consecutive hashes into one 32 bytes word so that the hashes becomes `[h1h0, h3h2, ...]`

In the output of `ethashproof`, all proofs of the elements are included in order in 1 array so that the proof
of dataset element 0 will be at the beginning of the array and element n's will be at the end. You will have to
determine the boundary of each proof yourself.
