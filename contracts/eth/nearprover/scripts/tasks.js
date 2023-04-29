const os = require('os');
const fs = require('fs');
const path = require('path');
const { upgrades, ethers } = require('hardhat');
const { defender } = require('hardhat');

function sleep (ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function verify (address) {
  if (hre.network.name !== 'hardhat' && hre.network.name !== 'localhost' && hre.network.name !== 'localnet') {
      let retry = 20;
      console.log('Sleeping before verification...');
      while ((await ethers.provider.getCode(address).catch(() => '')).length <= 3 && retry >= 0) {
          await sleep(5000);
          --retry;
      }
      await sleep(30000);

      await hre
          .run('verify:verify', {
              address,
          })
          .catch((error) => console.log(error));
  }
}
async function deployNearProverProxy (hre, args) {
    const { ethclientaddress, admin, pausedflags, upgrader} = args;

    if (!ethclientaddress) {
        throw new Error('ethclientaddress not set');
    }
    let [signer] = await ethers.getSigners();
    const NearProverFactory = (await hre.ethers.getContractFactory('NearProver')).connect(signer);
    const NearProver = await hre.upgrades.deployProxy(
        NearProverFactory,
        [ethclientaddress, pausedflags, upgrader],
        { kind: 'uups' },
    );
    await NearProver.deployed();

    // export proxy and implementation addresses to config.json
    const p = path.join(os.homedir(), '.rainbow/config.json');
    const cfg = fs.readFileSync(p);
    const rainbowConfig = JSON.parse(cfg);

    rainbowConfig.ethProverAddress = NearProver.address;
    rainbowConfig.ethProverImplementationAddress = await upgrades.erc1967.getImplementationAddress(
        NearProver.address,
    );

    let data = JSON.stringify(rainbowConfig, null, '\t');
    data = JSON.stringify(rainbowConfig, null, 2);
    fs.writeFileSync(p, data, { flags: 'w+' });
    await verify(rainbowConfig.ethProverAddress);
}

async function transferOwnership (currentadmin, newadmin, proveraddress) {
    const p = path.join(os.homedir(), '.rainbow/config.json');
    const cfg = fs.readFileSync(p);
    const rainbowConfig = JSON.parse(cfg);
    const wallet = new ethers.Wallet(rainbowConfig.ethMasterSk);

    if (currentadmin == wallet.address) {
        const NearProverFactory = await ethers.getContractFactory('NearProver');
        const NearProverContract = NearProverFactory.attach(proveraddress);
        const tx = await NearProverContract.transferOwnership(newadmin);
        const receipt = await tx.wait();
        if (receipt.status == 1) {
            console.log(
                `Transaction ${receipt.transactionHash} successfull: Ownership transferred from ${currentadmin} to ${newadmin}`,
            );
        } else {
            console.log(`Transaction ${receipt.transactionHash} failed`);
        }
    } else {
        console.log('Present owner is invalid');
    }
}

async function proposeUpgrade (proxyaddress, newcontractname, upgrader) {
    const newContract = await ethers.getContractFactory(newcontractname);
    console.log('Preparing proposal.....');
    const proposal = await defender.proposeUpgrade(proxyaddress, newContract, {multisig: upgrader, multisigType: 'Gnosis Safe'});
    console.log('Upgrade proposal created at: ', proposal.url);
}

module.exports = { deployNearProverProxy, transferOwnership, proposeUpgrade };
