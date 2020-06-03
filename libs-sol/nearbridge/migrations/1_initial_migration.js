const Migrations = artifacts.require('./Migrations.sol');
const Ed25519 = artifacts.require('./Ed25519.sol');
const NearBridge = artifacts.require('./NearBridge.sol');

module.exports = function (deployer) {
    deployer.deploy(Migrations);
    deployer.deploy(Ed25519);
    deployer.link(Ed25519, NearBridge);
    deployer.deploy(NearBridge);
};
