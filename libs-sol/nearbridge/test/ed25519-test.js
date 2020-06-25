"use strict";

const assert = require("assert").strict;
const {readFileSync} = require("fs");

const Web3 = require("web3");
const ganache = require("ganache-cli");

const abi = JSON.parse(readFileSync("../dist/Ed25519.full.abi"));
const code = readFileSync("../dist/Ed25519.full.bin", "utf8");
const tests = JSON.parse(readFileSync("ed25519-test-cases.json"));

(async () => {
  let web3 = new Web3(ganache.provider({gasLimit: 1e9}));
  let addr = (await web3.eth.personal.getAccounts())[0];
  let contract = await new web3.eth.Contract(abi).deploy({data: code}).send({from: addr, gas: 1e7});
  let goodMethod = null;
  let invocations = [];
  for (let {k, msg, sig, valid} of tests) {
    let [r, s] = [sig.substring(0, 64), sig.substring(64)];
    let [m1, m2] = [msg.substring(0, 64), msg.substring(64)];
    let method = contract.methods.check(`0x${k}`, `0x${r}`, `0x${s}`, `0x${m1}`, `0x${m2}`);
    invocations.push(method.call());
    if (!goodMethod && valid) {
      goodMethod = method;
    }
  }
  for (let i = 0; i < tests.length; i++) {
    if (tests[i].valid != await invocations[i]) {
      console.log(`Test failed: ${tests[i].description}`);
      process.exit(1);
    }
  }
  let receipt = await goodMethod.send({from: addr, gas: 1000000});
  console.log(`Gas used: ${receipt.gasUsed}`);
})();