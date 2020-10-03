# Migration Instructions

## From 1.x to 2.0.0

2.0.0 introduce three incompatible change in Ethereum and NEAR contracts: bridge-token-factory, generic token locker and TToken.

### Migration Instruction for your code

#### Token Factory on NEAR

Token factory is a new concept introduced in rainbow bridge lib/cli 2.0. It follows ERC721 that allow you to create multiple ERC20 tokens on NEAR. Back in 1.x, there was only one ERC token implemented as [mintable-fungible-token](https://github.com/near/rainbow-bridge-rs/tree/master/mintable-fungible-token) and only need initialize that. In 2.0, you need to first initalize the token factory contract

https://github.com/near/rainbow-bridge-lib/blob/master/init/near-token-factory.js#L74

And then deploy a erc20 token in factory:

https://github.com/near/rainbow-bridge-lib/blob/master/transfer-eth-erc20/deploy-token.js#L76

After that deployed erc20 token is very similar to 1.x mintable-fungible-token, you can deposit (renamed from mint, usage is same) or withdraw (renamed from burn, usage is same) it.

#### Token Locker on Ethereum

Token locker soldity contract has changed from:

```
constructor(IERC20 ethToken, bytes memory nearToken, INearProver prover) public;
function lockToken(uint256 amount, string memory accountId) public;
function unlockToken(bytes memory proofData, uint64 proofBlockHeight) public;
```

to:

```
constructor(bytes memory nearTokenFactory, INearProver prover) public;
function lockToken(IERC20 token, uint256 amount, string memory accountId) public;
function unlockToken(bytes memory proofData, uint64 proofBlockHeader) public;
```

You will need to call updated method. Basically, token locker in rainbow bridge lib/cli 1.x can only lock one kind of token,
specified when initialized the locker. Locker in rainbow bridge 2.0 (provided by [rainbow-token-connector](https://github.com/near/rainbow-token-connector)) can lock and unlock any erc20 token created in `nearTokenFactory`. Therefore when locking, which token to lock is required parameter.

#### TToken on Ethereum

[MyERC20.sol](https://github.com/near/rainbow-bridge-sol/blob/a3968cee82f2923aee9fbe2387b7045993eafc0f/token-locker/contracts/MyERC20.sol) in rainbow bridge 1.0 has been replaced with [TToken.sol](https://github.com/near/rainbow-token-connector/blob/master/erc20-connector/contracts/test/TToken.sol). This is a compatible change.

### Migration Instruction to Use NEAR Deployed Contracts and Bridge Services

You only need to update the config file to use new contract addresses, see [Using Bridge on Testnet](README.md#using-bridge-on-testnet) and look for `rainbow-bridge-cli 2.x`

### Migration Instruction for Deployment

If you are deploying your own bridge, eth2near relayer, near2eth relayer, eth ed25519, eth client, eth prover, near client and near prover can be reused. you need to remove these lines from your `~/.rainbow/config.json`:

```
    "ethErc20Address": "...",
    "ethLockerAddress": "...",
    "nearFunTokenAccount": "..."
```

And add this line:

```
	"nearTokenFactoryAccount": "fill a non exist near account in your namespace, going to be created as your token factory account",
```

Then redeploy these contracts:

```
rainbow init-eth-erc20 # use TToken
rainbow init-eth-locker # use new generic locker
rainbow init-near-token-factory # use token factory
```

You should be able to use bridge again with same transfer from near and to near command as before!
