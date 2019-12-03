const Web3 = require('web3');
const nearlib = require('nearlib');

(async function () {

    const web3 = new Web3("https://mainnet.infura.io/v3/b5f870422ee5454fb11937e947154cd2");
    const near = await nearlib.connect({
        nodeUrl: 'https://34.67.252.132'
    });

    console.log(await web3.eth.getBlockNumber());
    console.log(nearlib);
})()
