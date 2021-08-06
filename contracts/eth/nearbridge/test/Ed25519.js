const { assert } = require('chai');

describe('Ed25519', () => {
  let Ed25519;

  before(async () => {
    Ed25519 = await (await ethers.getContractFactory('Ed25519')).deploy();
  });

  for (const { description, k, msg, sig, valid } of require('./ed25519-tests.json')) {
    it(description, async () => {
        const [r, s] = [sig.substring(0, 64), sig.substring(64)];
        const [m1, m2] = [msg.substring(0, 64), msg.substring(64)];
        assert.equal(await Ed25519.check(`0x${k}`, `0x${r}`, `0x${s}`, `0x${m1}`, `0x${m2}`), valid);
    });
  }
});
