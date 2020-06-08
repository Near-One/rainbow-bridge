const Migrations = artifacts.require('./Migrations.sol');
const Ed25519 = artifacts.require('./Ed25519.sol');
const NearBridge = artifacts.require('./NearBridge.sol');

module.exports = async function (deployer) {
    await deployer.deploy(Migrations);
    await deployer.deploy(Ed25519);
    await deployer.deploy(NearBridge, (await Ed25519.deployed()).address);
};
