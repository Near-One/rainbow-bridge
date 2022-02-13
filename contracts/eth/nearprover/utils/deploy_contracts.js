const { ethers } = require('hardhat');

async function main() {
    [deployerAccount] = await ethers.getSigners();

    console.log(`Deploying contracts with the account: ${deployerAccount.address}`);

    const accountBalance = await deployerAccount.getBalance();
    console.log(`Account balance: ${accountBalance} wei`);
    console.log(`Account balance: ${ethers.utils.formatEther(accountBalance)} ETH`);

    // Make the deployer admin
    const adminAccount = deployerAccount;

    const nearBridgeMock = await (await ethers.getContractFactory('NearBridgeMock')).deploy();
    await nearBridgeMock.deployed();
    console.log(`Deployed bridge mock to: ${nearBridgeMock.address}`);

    const nearProverContractFactory = await ethers.getContractFactory('NearProver')
    const nearProver = await nearProverContractFactory.deploy(
        nearBridgeMock.address,
        adminAccount.address,
        0
    );

    await nearProver.deployed();
    console.log(`Near Prover deployed to: ${nearProver.address}`);
}

main()
    .then(() => process.exit(0))
    .catch(error => {
        console.error(error);
        process.exit(1);
    });
