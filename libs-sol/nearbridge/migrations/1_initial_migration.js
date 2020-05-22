const Migrations = artifacts.require('./Migrations.sol');
const NearBridge = artifacts.require('./NearBridge.sol');

module.exports = function (deployer) {
    deployer.deploy(Migrations);
    deployer.deploy(NearBridge, "0x", "0x");
};
