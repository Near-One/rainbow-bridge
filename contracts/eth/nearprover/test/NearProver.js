const { expect } = require('chai');
const { ethers, upgrades } = require('hardhat');
const { borshifyOutcomeProof } = require(`rainbow-bridge-lib/rainbow/borsh`);
const fs = require('fs').promises;
const { computeMerkleRoot } = require('../utils/utils');

let NearProver, NearBridgeMock, accounts;

beforeEach(async function () {
    accounts = await ethers.getSigners();
    NearBridgeMock = await (await ethers.getContractFactory('NearBridgeMock')).deploy();
    const NearProverFactory = await ethers.getContractFactory('NearProver');
    NearProver = await upgrades.deployProxy(NearProverFactory, [
        NearBridgeMock.address,
        0,
    ], { kind: 'uups' });
    await NearProver.deployed();
});

async function testProof(merkleRoot, height, proofPath) {
    let proof = require(proofPath);
    console.log(computeMerkleRoot(proof).toString('hex'));
    expect(merkleRoot === '0x' + computeMerkleRoot(proof).toString('hex')).to.be.true;
    proof = borshifyOutcomeProof(proof);
    await NearBridgeMock.setBlockMerkleRoot(height, merkleRoot);
    expect(await NearProver.proveOutcome(proof, height)).to.be.true;
}

it('upgarde the proxy', async function () {
    const NearProverV2Factory = await ethers.getContractFactory('NearProverV2');
    NearProver = await upgrades.upgradeProxy(NearProver.address, NearProverV2Factory);
    expect(await NearProver.version()).eq('2.0.0');
});

it('transfer contract ownership', async function () {
    expect(await NearProver.transferOwnership(accounts[1].address))
        .emit(NearProver, 'OwnershipTransferred')
        .withArgs(accounts[0].address, accounts[1].address);

    const pauseRole = await NearProver.PAUSE_ROLE();
    const adminRole = await NearProver.DEFAULT_ADMIN_ROLE();
    expect(await NearProver.hasRole(pauseRole, accounts[1].address)).ok;
    expect(await NearProver.hasRole(adminRole, accounts[1].address)).ok;

    expect(await NearProver.hasRole(pauseRole, accounts[0].address)).not.ok;
    expect(await NearProver.hasRole(adminRole, accounts[0].address)).not.ok;
});

it('Fail to upgrade contract, Unauthorized', async function () {
    expect(await NearProver.transferOwnership(accounts[1].address))
        .emit(NearProver, 'OwnershipTransferred')
        .withArgs(accounts[0].address, accounts[1].address);
    
    const NearProverV2Factory = await ethers.getContractFactory('NearProverV2');
    await expect(upgrades.upgradeProxy(NearProver.address, NearProverV2Factory))
        .reverted;
});

it('should be ok', async function () {
    await testProof('0x22f00dd154366d758cd3e4fe81c1caed8e0db6227fe4b2b52a8e5a468aa0a723', 498, './proof2.json');
    await testProof('0x0d0776820a9a81481a559c36fd5d69c33718fb7d7fd3be7564a446e043e2cb35', 1705, './proof3.json');
    await testProof('0x1f7129496c461c058fb3daf258d89bf7dacb4efad5742351f66098a00bb6fa53', 5563, './proof4.json');
    await testProof('0xa9cd8eb4dd92ba5f2fef47d68e1d73ac8c57047959f6f8a2dcc664419e74e4b8', 384, './proof5.json');
    await testProof('0xcc3954a51b7c1a86861df8809f79c2bf839741e3e380e28360b8b3970a5d90bd', 377, './proof6.json');
});

if (process.env['NEAR_PROOFS_DIR']) {
    it('should able to verify proofs from dump', async function () {
        this.timeout(0);
        let proofFiles = await fs.readdir(process.env['NEAR_PROOFS_DIR']);

        for (let i = 0; i < proofFiles.length; i++) {
            let proof = require(process.env['NEAR_PROOFS_DIR'] + '/' + proofFiles[i]);
            const height = proof.block_header_lite.inner_lite.height;
            await NearBridgeMock.setBlockMerkleRoot(height, '0x' + computeMerkleRoot(proof).toString('hex'));
            proof = borshifyOutcomeProof(proof);
            expect(await NearProver.proveOutcome(proof, height)).to.be.true;
            console.log('proved proof ' + proofFiles[i]);
        }
    });
}

describe('NearProver with admin access', () => {
    const BRIDGE_ADDRESS_SLOT = 2;

    beforeEach(async () => {
        [deployerAccount] = await ethers.getSigners();

        // Make the deployer admin
        adminAccount = deployerAccount;

        nearBridgeMock = await (await ethers.getContractFactory('NearBridgeMock')).deploy();
        nearProver = await (await ethers.getContractFactory('NearProver')).deploy(
            nearBridgeMock.address,
            adminAccount.address,
            0
        );

    });

    describe('Upgradability', async () => {
        it('should upgrade the bridge address', async () => {
            const initialBridgeAddress = await nearProver.bridge();
            expect(initialBridgeAddress)
                .to
                .equal(nearBridgeMock.address);

            const newBridge = await (await ethers.getContractFactory('NearBridgeMock')).deploy();
            expect(await newBridge.address)
                .to
                .not
                .equal(initialBridgeAddress);

            // Mask matches only on the latest 20 bytes (to store the address)
            const mask = ethers.BigNumber.from("0x000000000000000000000000ffffffffffffffffffffffffffffffffffffffff");
            nearProver.adminSstoreWithMask(BRIDGE_ADDRESS_SLOT, newBridge.address, mask);

            expect(await nearProver.bridge())
                .to
                .equal(newBridge.address);
        });

        it('should upgrade the bridge address from the provided hex string', async () => {
            const initialBridgeAddress = await nearProver.bridge();
            expect(initialBridgeAddress)
                .to
                .equal(nearBridgeMock.address);

            const newBridgeAddress = '0x891B2749238B27fF58e951088e55b04de71Dc374';
            const newBridgeAddressBN = ethers.BigNumber.from(newBridgeAddress);

            const mask = ethers.BigNumber.from("0x000000000000000000000000ffffffffffffffffffffffffffffffffffffffff");
            nearProver.adminSstoreWithMask(BRIDGE_ADDRESS_SLOT, newBridgeAddressBN, mask);

            expect(await nearProver.bridge())
                .to
                .equal(newBridgeAddress);
        });
    });
});
