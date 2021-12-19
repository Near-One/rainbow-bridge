const Web3 = require('web3');
const createLedgerSubprovider = require('@ledgerhq/web3-subprovider').default;
const hid = require('@ledgerhq/hw-transport-node-hid').default;
const ProviderEngine = require('web3-provider-engine');
const RpcSubprovider = require('web3-provider-engine/subproviders/rpc');

require('dotenv').config();

const { ethers } = require('hardhat');
const { assert } = require('chai');
const BRIDGE_ADDRESS_SLOT = 2;

async function upgradeProversBridgeAddressTo (provider, proverAddress, newBridgeAddress, ledgerKeyPath) {
    const network = 'goerli';
    const networkId = 5;
    const engine = new ProviderEngine();
    const getTransport = () => hid.create();
    const ledger = createLedgerSubprovider(getTransport, {
        networkId: networkId,
        accountsLength: 5,
        paths: [ledgerKeyPath],
    });

    engine.addProvider(ledger);
    const rpcUrl = `https://${network}.infura.io/v3/${process.env.INFURA_API_KEY}`;
    console.log(rpcUrl);
    engine.addProvider(new RpcSubprovider({ rpcUrl }));
    engine.start();
    const web3 = new Web3(engine);
    const nearProverFactory = await ethers.getContractFactory('NearProver');
    const nearProver = nearProverFactory.attach(proverAddress);

    console.log(`Got prover at address: ${proverAddress}`);

    const initialBridgeAddress = await nearProver.bridge();
    console.log(`Initial bridge address: ${initialBridgeAddress}`);
    console.log(`Trying to upgrade bridge address to: ${newBridgeAddress}`);
    const abi = require('../artifacts/contracts/NearProver.sol/NearProver.json').abi;
    const accounts = await web3.eth.getAccounts();
    const signerAccount = accounts[0];
    console.log(signerAccount);
    console.log(`Used account: ${signerAccount}`);
    const web3Contract = new web3.eth.Contract(abi, proverAddress);

    assert.equal(
        await signerAccount,
        await nearProver.admin(),
        'The used account is not an admin of NearProver',
    );

    // Mask matches only on the latest 20 bytes (to store the address)
    const mask = '0x000000000000000000000000ffffffffffffffffffffffffffffffffffffffff';
    const options = {
        from: signerAccount,
        gas: 50000,
        gasPrice: 150000000000, // 150 Gwei
    };
    const response = await web3Contract.methods.adminSstoreWithMask(BRIDGE_ADDRESS_SLOT, newBridgeAddress, mask).send(options);
    console.log(response);
}

async function getProversBridgeAddress (proverAddress) {
    const nearProverFactory = await ethers.getContractFactory('NearProver');
    const nearProver = nearProverFactory.attach(proverAddress);
    const bridgeAddress = await nearProver.bridge();

    return bridgeAddress;
}

exports.upgradeProversBridgeAddressTo = upgradeProversBridgeAddressTo;
exports.getProversBridgeAddress = getProversBridgeAddress;
