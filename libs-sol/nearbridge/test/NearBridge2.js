
const { time } = require('@openzeppelin/test-helpers');
const bs58 = require('bs58');
const {borshify} = require('../../../environment/lib/near2eth-relay');
const abiDecoder = require('abi-decoder');

const Ed25519 = artifacts.require('Ed25519');
const NearBridge = artifacts.require('NearBridge');
const NearDecoder = artifacts.require('NearDecoder');

async function timeIncreaseTo (seconds) {
    const delay = 1000 - new Date().getMilliseconds();
    await new Promise(resolve => setTimeout(resolve, delay));
    await time.increaseTo(seconds);
}

// function borshify (block) {
//     return Buffer.concat([
//         bs58.decode(block.prev_block_hash),
//         bs58.decode(block.next_block_inner_hash),
//         Buffer.concat([
//             web3.utils.toBN(block.inner_lite.height).toBuffer('le', 8),
//             bs58.decode(block.inner_lite.epoch_id),
//             bs58.decode(block.inner_lite.next_epoch_id),
//             bs58.decode(block.inner_lite.prev_state_root),
//             bs58.decode(block.inner_lite.outcome_root),
//             web3.utils.toBN(block.inner_lite.timestamp).toBuffer('le', 8),
//             bs58.decode(block.inner_lite.next_bp_hash),
//             bs58.decode(block.inner_lite.block_merkle_root),
//         ]),
//         bs58.decode(block.inner_rest_hash),
//
//         Buffer.from([1]),
//         web3.utils.toBN(block.next_bps.length).toBuffer('le', 4),
//         Buffer.concat(
//             block.next_bps.map(nextBp => Buffer.concat([
//                 web3.utils.toBN(nextBp.account_id.length).toBuffer('le', 4),
//                 Buffer.from(nextBp.account_id),
//                 nextBp.public_key.substr(0, 8) === 'ed25519:' ? Buffer.from([0]) : Buffer.from([1]),
//                 bs58.decode(nextBp.public_key.substr(8)),
//                 web3.utils.toBN(nextBp.stake).toBuffer('le', 16),
//             ])),
//         ),
//
//         web3.utils.toBN(block.approvals_after_next.length).toBuffer('le', 4),
//         Buffer.concat(
//             block.approvals_after_next.map(
//                 signature => signature === null
//                     ? Buffer.from([0])
//                     : Buffer.concat([
//                         Buffer.from([1]),
//                         signature.substr(0, 8) === 'ed25519:' ? Buffer.from([0]) : Buffer.from([1]),
//                         bs58.decode(signature.substr(8)),
//                     ]),
//             ),
//         ),
//     ]);
// }

// contract('NearBridge2', function ([_, addr1]) {
//     beforeEach(async function () {
//         this.decoder = await NearDecoder.new();
//         this.bridge = await NearBridge.new((await Ed25519.deployed()).address, web3.utils.toBN(1e18), web3.utils.toBN(10));
//         await this.bridge.deposit({ value: web3.utils.toWei('1') });
//     });
//
//     it('should be ok', async function () {
//         const block9605 = borshify(require('./block_9605.json'));
//         const block9610 = borshify(require('./block_9610.json'));
//
//         await this.bridge.initWithBlock(block9605);
//         await this.bridge.blockHashes(9605);
//         expect(await this.bridge.blockHashes(9605)).to.be.equal(
//             '0xc4770276d5e782d847ea3ce0674894a572df3ea75b960ff57d66395df0eb2a34',
//         );
//
//         await this.bridge.addLightClientBlock(block9610);
//         expect(await this.bridge.blockHashes(9610)).to.be.equal(
//             '0xf28629da269e59f2494c6bf283e9e67dadaa1c1f753607650d21e5e5b916a0dc',
//         );
//     });
// });

// contract('NearBridge3', function ([_, addr1]) {
//     beforeEach(async function () {
//
//     });
//
//     it('should be ok', async function () {
//         this.decoder = await NearDecoder.new();
//         this.bridge = await NearBridge.new((await Ed25519.deployed()).address, web3.utils.toBN(1e18), web3.utils.toBN(3600));
//         await this.bridge.deposit({value: web3.utils.toWei('1')});
//
//         const block9580503 = require('./block_9580503.json');
//         const block9580534 = require('./block_9580534.json');
//         const block9580624 = require('./block_9580624.json');
//
//         await this.bridge.initWithBlock(borshify(block9580503));
//         await this.bridge.blockHashes(9580503);
//
//         await this.bridge.addLightClientBlock(borshify(block9580534));
//         await this.bridge.blockHashes(9580534);
//
//         console.log("Verifying block 9580534");
//         for (let i = 0; i < block9580534.approvals_after_next.length; i++) {
//             if (block9580534.approvals_after_next[i]) {
//                 if (await this.bridge.checkBlockProducerSignatureInLastBlock(i)) {
//                     console.log(`Signature ${i} is OK`);
//                 } else {
//                     console.log(`Signature ${i} is NOT OK`);
//                 }
//             }
//         }
//
//         const now = await time.latest();
//         await timeIncreaseTo(now.add(time.duration.seconds(3600)));
//
//         await this.bridge.addLightClientBlock(borshify(block9580624));
//         await this.bridge.blockHashes(9580624);
//
//         for (let i = 0; i < block9580624.approvals_after_next.length; i++) {
//             if (block9580624.approvals_after_next[i]) {
//                 if (await this.bridge.checkBlockProducerSignatureInLastBlock(i)) {
//                     console.log(`Signature ${i} is OK`);
//                 } else {
//                     console.log(`Signature ${i} is NOT OK`);
//                 }
//             }
//         }
//     });
// });

contract('NearBridge4', function ([_, addr1]) {
    beforeEach(async function () {

    });

    it('should be ok', async function () {
        // See https://ropsten.etherscan.io/txs?a=0x276d4d74dc14251c8d75ff4ae9175142e1c2254d&ps=100&p=21
        // for the transactions submitted to the contract and to know the moment it started failing.
        this.decoder = await NearDecoder.new();
        this.bridge = await NearBridge.new((await Ed25519.deployed()).address, web3.utils.toBN(1e18), web3.utils.toBN(3600));
        await this.bridge.deposit({value: web3.utils.toWei('1')});
        const testABI = [{"constant":false,"inputs":[{"internalType":"bytes","name":"data","type":"bytes"}],"name":"addLightClientBlock","outputs":[],"payable":true,"stateMutability":"payable","type":"function"}];
        abiDecoder.addABI(testABI);

        const block9657410 = require('./block_9657410.json');
        // Make sure when we borshify the input in the JSON file it is exatly the same as it was passed in
        // https://ropsten.etherscan.io/tx/0x911efd070a6f3fa0696ea90d287d1cb4d134df5655fc9ecb80a18b5dec43880c
        const raw_9657410 = '0x6d2d6ae000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000753944a379677f1ffa3d5f344f46f4ac259618df730ff8b7ca0c304815298c8c018ee867d97a059992446e4416936f594a6c0f2a5d7d9013e8a8f86397834d3ff7f425c93000000000061f7c27bc7f6b2c7cf77d9dd1af0b7c86f749c24b2a5014fa5953b5735ad955387aacffa54853f6fc0878c8b41ec7dbdaca88760136208999bbd8ec257a055a2166486a1e239da565bdb0798a31f3fe7998b33713393b3d57d710bf10013949a66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f29256eed9ee976d02116afd62c70f100afcffcc90fee0770e42dc159029fa7430bdd25ebbb796054009f750ce1d274d80e35d028de5164adcab132e6ea9f45785cf7b44a3ba2ecc184b80bbeb4646408ca89d7aab4f7602c293a5fc66a6b979f5717cdce2916deb657d3010c000000150000006365727475736f6e652e7374616b696e67706f6f6c00aa1cebee2f9edca96c4d47735009a7712a4a86023157a7d6c6dfadc7a25046d5c17f8241184f17a0d3b3791a0200000012000000696e6f74656c2e7374616b696e67706f6f6c004867392e5416066dae09ce0166b3568f827cbe1bee825022cbf062703ac1f116f2d2773b9518364afec541e9000000001600000062617a696c696b7375622e7374616b696e67706f6f6c0029cf97077ac0903d33a922a1f37472a9af5464f4aefd09e3eba16f6a00f75c6f9fcecca2a361faeb6baad9bb00000000170000006269736f6e747261696c732e7374616b696e67706f6f6c0063ef6a7797dd9752934ec88efde64918714efb55800bc09ac193eed2815ba92939b0e57590835492fd29f9a70100000014000000637279707469756d2e7374616b696e67706f6f6c001c66f71a103f9c0f32e824432631b36be29cd6a3671d4f989f757abd622cfd0095cfa03b54e189095b6d967b030000001d000000706f6f6c5f646f6b69616361706974616c2e7374616b696e67706f6f6c00776584afeb29ab568c4a0ef3093f35c72869a75c375f9033784d940b57f2bbe2c18299d6fce245fb236cc226010000001b0000007374616b65706f6f6c2e68617368717561726b2e746573746e65740076157290c20c9ff7bb215e82331e40eb2b5facd8734e6f6e02d63406c082d321ad1e3db780f09b6d0b66d53301000000180000007374616b696e672e647372766c6162732e746573746e657400d28ae5ae331f4690461cf7d9b9b7b2782942a0c5986a18e64d7b0d39f7598fabbcf81eff862d9728acc6a2c9000000000f000000746f702e7374616b696e67706f6f6c0042e798e80d92ec3cadbd2270153d40df36c1af741e6e61cb6b2beef33d3cebf459aa4b2454041eebaa5218c7000000001600000066726573686e656172732e7374616b696e67706f6f6c005c0e6775d442506f04ec07832de1b20303020a1ee2780548459306260a14bb0b5d9ff2e20bcfc7f4030aa0d6000000001b0000006275696c646c696e6b735f706f6f6c2e7374616b696e67706f6f6c00bc44bb85379ee13bfa3555f361e8069f10f70e03515f63f1c209faeb569cb56774ababacd8976c6202237ae800000000120000007374616b65642e7374616b696e67706f6f6c0042ccfc75738eedf38a9cfd5d19c06f0db8dedd161f4e3496b9d8cb63c3bd72b3a6e39e1007a67c79411fc53e010000000a0000000100b567bf4de24c1a12e8fa5b49ce9d5d61ef965590f71d871f887f1eb5f6f5dc4c4267d9c0ce0775e0ebac58a07752373ade5593df90ddaa61316e725c823c77050100a9a30cb48f778a90a5e49226734367567d4b73a48253d976df266b4e51771d9334366fe3ca5d68b1e40028237d3f8f52b886699e363ab1b02193977c6c9d9a020100b82c1d69c6a5c0c7696bd882aa7045daeb09876db2d776be8cc31c97258e1545a056c07a9e0fb0fdf64d5e8cc7a114e31a507f1eec652f7ccfa1b8afedf5ec0701007ddd0d787c7325a7d90dc8365aef72292ac49c678c4daeeb3b5c0bcce545603b0aa812738d7ac9cb55dc7afb46f06e523f9cf3b21e24a80c9505340e2070010901003484fee60ded2f3c884d6a6458a1c6d3d7cf9c2f6dd21fbbb5c0beded6ecf47f44fc8c3192d47fef55c396731353cacbf6c54ea60fbd37692b7f7cc944d4fe0f010000754c761732e37a69279c8ae312760702b528753d4a7b76c0142035348c39c06623f39fb5d5018ebae03d34814bc946e30a737f469f3d2ece090cb07f4ea80d0100f51c87a2b568904008d6946fb493b6b92564e42970475cbc9041ee90068ffbafef7b23a75f006a7bf11a01614db5e751d610253d905737fc883f62de847b470f01009c42d297796f9cc654d09c960c1d4f02d78751bf1e1db9632b27169ebda8f5ba28d03f231fe9296c55a6d5e07cb67b1fc3b21b9eddc7fcbe646e6f87bbeb9205010023146b6f59e561324e240642d7265359b8e71f9cefcf51cbd98a70da0295dd2c62a78db1e49a6feb7e240d39777ddb1f56d8a87c7afda5149056766ca2c1d40e0100058370b8911e948a745e352609e712598a1ba41e3242fd04ba5f170ed0c81868ca8c135195f289229393886354ca7456ef1243d39575d1658cd8a212d03e9c0c00000000000000000000000000';
        const borshified_real_9657410 = abiDecoder.decodeMethod(raw_9657410).params[0].value;
        expect(borshified_real_9657410).to.be.equal('0x'+borshify(block9657410).toString('hex'));

        const block9657547 = require('./block_9657547.json');
        // Make sure when we borshify the input in the JSON file it is exatly the same as it was passed in
        // https://ropsten.etherscan.io/tx/0xa788c07600534783c277f1d2b36b43df63755f37b5f7992daea0ad7d568526c0
        const raw_9657547 = '0x6d2d6ae0000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000007124f7980cec4115b0d8f2760e70523f84ac7db34fc33272a0c04987b47a1c61ccc94202239a96433e24af4ae7d78d8e3a7b462e805bf091933e9d9eb1452d044e7cb5c93000000000061f7c27bc7f6b2c7cf77d9dd1af0b7c86f749c24b2a5014fa5953b5735ad955387aacffa54853f6fc0878c8b41ec7dbdaca88760136208999bbd8ec257a055a283bac419b68232e7c8439c8f9d730ba8d5ca33956f620716fed85bfedef9819566687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f292544822c9191d02116afd62c70f100afcffcc90fee0770e42dc159029fa7430bdd25ebbb796054009f948e3e6f5265d1b542c0612ad16fb602b7160ce0730a8b8086b650dbffe2fd5348068bbf95aaef0b3e6829bd4e5f44fd2dc4edad6dd0c0a591a8bc1446a5fc75010c000000150000006365727475736f6e652e7374616b696e67706f6f6c00aa1cebee2f9edca96c4d47735009a7712a4a86023157a7d6c6dfadc7a25046d5c17f8241184f17a0d3b3791a0200000012000000696e6f74656c2e7374616b696e67706f6f6c004867392e5416066dae09ce0166b3568f827cbe1bee825022cbf062703ac1f116f2d2773b9518364afec541e9000000001600000062617a696c696b7375622e7374616b696e67706f6f6c0029cf97077ac0903d33a922a1f37472a9af5464f4aefd09e3eba16f6a00f75c6f9fcecca2a361faeb6baad9bb00000000170000006269736f6e747261696c732e7374616b696e67706f6f6c0063ef6a7797dd9752934ec88efde64918714efb55800bc09ac193eed2815ba92939b0e57590835492fd29f9a70100000014000000637279707469756d2e7374616b696e67706f6f6c001c66f71a103f9c0f32e824432631b36be29cd6a3671d4f989f757abd622cfd0095cfa03b54e189095b6d967b030000001d000000706f6f6c5f646f6b69616361706974616c2e7374616b696e67706f6f6c00776584afeb29ab568c4a0ef3093f35c72869a75c375f9033784d940b57f2bbe2c18299d6fce245fb236cc226010000001b0000007374616b65706f6f6c2e68617368717561726b2e746573746e65740076157290c20c9ff7bb215e82331e40eb2b5facd8734e6f6e02d63406c082d321ad1e3db780f09b6d0b66d53301000000180000007374616b696e672e647372766c6162732e746573746e657400d28ae5ae331f4690461cf7d9b9b7b2782942a0c5986a18e64d7b0d39f7598fabbcf81eff862d9728acc6a2c9000000000f000000746f702e7374616b696e67706f6f6c0042e798e80d92ec3cadbd2270153d40df36c1af741e6e61cb6b2beef33d3cebf459aa4b2454041eebaa5218c7000000001600000066726573686e656172732e7374616b696e67706f6f6c005c0e6775d442506f04ec07832de1b20303020a1ee2780548459306260a14bb0b5d9ff2e20bcfc7f4030aa0d6000000001b0000006275696c646c696e6b735f706f6f6c2e7374616b696e67706f6f6c00bc44bb85379ee13bfa3555f361e8069f10f70e03515f63f1c209faeb569cb56774ababacd8976c6202237ae800000000120000007374616b65642e7374616b696e67706f6f6c0042ccfc75738eedf38a9cfd5d19c06f0db8dedd161f4e3496b9d8cb63c3bd72b3a6e39e1007a67c79411fc53e010000000a00000001007de1177712eeff677fc00931cc39b4c360f885bd1cd46787b0388473924dd075d35c2064e7924290b48a5702b643b17283eb1d5fc3d438ca9758683f9050eb0001008d281644b70581ea39df72995657860b90b1cb321e37c760f2cdb2388b54b8f062c1b99ac36c3f9a75d037ad7e71f6087b48affaa4d59ef86a03d2e00f264e0201000a8313186c74b613427b6d10378a0d56f899507aa6c728cdc8f37c74edfd8da71383576ef4edde2f445d8cf9d62d6a21167eb4511160928fecd3d1c669dc5c0501005a2f7a4b0e6c69e1a0e2c3342e0f985c1b3b7785f3128e5e022097417595dd2b17c430877765418c0ab316920648c4ce4e38b605f00222879885ad519fa31607000100878c4cf13de2c372d821fc6cd965c9d6d7de541d5cb7e64f0db98d85b3d6744b4db39f9d5a0e64fec19ee5a8ba4c6493894f21138582165f325e95404464130d01006bfb11c25c2495df15f60fb3214384f178e33194c681b804547f08c9ed152278f12c03ed9c1d2f12e0406bcc596cab5842dbac7f29021c9e35029fa0df3efe060100a7422daf1a53cbddd6b2f9afd020a6eb808d5565246c080d70120d4de7b3861e4dc2891386c93ee7826f2e3b7a681e425b75c45e8a69abbc4c494972a94fb10101001a0def02d8a9819982fb18e03e7571061815b67346a9eedaabe00db74c7e2738f63ded80748b0778688e03892db390b0a5074d848c6b6f4184db4ca0ecd0530c01000cd2f60f5ab2b89c55c36075960082ead69e77e9b5dca87558fa9f65db98fb85e1dc9e182fec2c065c7e5c57d5027e2d072f3918822018850f528c5329c57e0f0000000000000000000000000000';
        const borshified_real_9657547 = abiDecoder.decodeMethod(raw_9657547).params[0].value;
        expect(borshified_real_9657547).to.be.equal('0x'+borshify(block9657547).toString('hex'));

        const block9657674 = require('./block_9657674.json');
        // Make sure when we borshify the input in the JSON file it is exatly the same as it was passed in
        //
        const raw_9657674 = '0x6d2d6ae000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000755439961cf73d3d14e88d24009c8cb5a208612b2ef55f8eb026288cdb48dc9840c12ae92dd5196d0c739b2ffa7dd4624e0173fd2a6b80806449bc63760539efb974a5d93000000000087aacffa54853f6fc0878c8b41ec7dbdaca88760136208999bbd8ec257a055a2b3a8307732207a8e7a9d6cc8e459d65af9d62847673e7ce2be6362d2530cfc099471075d5182777361694d4aa3206f7ff422deda33fb464fdf67c2709ee5791266687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f29252000026faad02116218ab30863f92b8656e5885a52e858a85019cfef309b76959cd05067f09a43506e0ff556393ea7f8264924b47517fc1c8722d6b08e58b5d562dac95153a36dddf3e8d114a9f75a97d55b4deeae9cb8b29461f9b5897a58aaface98a116bec785010c00000014000000637279707469756d2e7374616b696e67706f6f6c001c66f71a103f9c0f32e824432631b36be29cd6a3671d4f989f757abd622cfd0048612dfb19abd4cc2e7ca17103000000120000007374616b65642e7374616b696e67706f6f6c0042ccfc75738eedf38a9cfd5d19c06f0db8dedd161f4e3496b9d8cb63c3bd72b3ecd621c91b93a8d1b4c19951010000001600000066726573686e656172732e7374616b696e67706f6f6c005c0e6775d442506f04ec07832de1b20303020a1ee2780548459306260a14bb0b5d9ff2e20bcfc7f4030aa0d600000000170000006269736f6e747261696c732e7374616b696e67706f6f6c0063ef6a7797dd9752934ec88efde64918714efb55800bc09ac193eed2815ba92939b0e57590835492fd29f9a7010000001d000000706f6f6c5f646f6b69616361706974616c2e7374616b696e67706f6f6c00776584afeb29ab568c4a0ef3093f35c72869a75c375f9033784d940b57f2bbe2662bd80b92489fe14fbcbf38010000000f000000746f702e7374616b696e67706f6f6c0042e798e80d92ec3cadbd2270153d40df36c1af741e6e61cb6b2beef33d3cebf464e75b21da4c85a9639f48d800000000150000006365727475736f6e652e7374616b696e67706f6f6c00aa1cebee2f9edca96c4d47735009a7712a4a86023157a7d6c6dfadc7a25046d5099c0d6d4bb0164317e89633020000001600000062617a696c696b7375622e7374616b696e67706f6f6c0029cf97077ac0903d33a922a1f37472a9af5464f4aefd09e3eba16f6a00f75c6ffb7ed2537295ca6cbdf5e7c7000000001b0000006275696c646c696e6b735f706f6f6c2e7374616b696e67706f6f6c00bc44bb85379ee13bfa3555f361e8069f10f70e03515f63f1c209faeb569cb5672fd58a1a4000dee94f10a5f600000000180000007374616b696e672e647372766c6162732e746573746e657400d28ae5ae331f4690461cf7d9b9b7b2782942a0c5986a18e64d7b0d39f7598fab432c8aa2b19e84c6b08ebbd5000000001b0000007374616b65706f6f6c2e68617368717561726b2e746573746e65740076157290c20c9ff7bb215e82331e40eb2b5facd8734e6f6e02d63406c082d321e31e5db901879f5784f38a460100000012000000696e6f74656c2e7374616b696e67706f6f6c004867392e5416066dae09ce0166b3568f827cbe1bee825022cbf062703ac1f116dac90a2610a4a4d14bb36cf7000000000c0000000100fe3e9ae6d6578b69187614fdd0d2752d010fd366375f8a8731f8844dfcf23a6f09668b270c8a65992b88a7a22d70862e1b16c627ba0007f76a46816f84a9230e000100f1df3a5208e61a9dcb05c0de64632fbea8de790f412c947bf7cdb5d1d559e076c074ce8d1a90d3b584283490dca1a54ad97e980ae13d47b9be75afa3d12f7f090100f66e20240157ad10cceed6232e51098b7dcccb2082c166b061b6c02ac05ab87c5a596adcb7999b6edbcd4e97b530dae226c49b5a57af8e2b9e352abe87c4c90f010025bda1f12301c3999cfadaa9c9baf8f0cd1063824f4c045f576cdf803eb252a2658062d481abdb6f1aad45fda002be8db34a39076ae6880de8d6c4fb062ea50d0001000c466664a6e631cda589b72ca591b642006dad75d83500b1e9cf6d4e99cb338765f2f7557c1777dbf076089166b423a9ba2fb69fc9b6e07531b916b19190cd0101002f8f63954a3b014f8c3996e2ec3835fe24581a4c024afa7e401cdfe4caed09e26133c5d5527327b1f8e1a484062c1cfdcff13ee1d19f49ade499ee2bc3a7d6070100bca383dd0f870aa4c2ec6b13f80226a9d6324544373df64ab237b72e1c9edc1d6051ccb9d8e5a0c2c4b21e10d2864ff65a9c055a96706207fdfefde9e453e00a0100439f39a4b786c01360443b396265c937c97a4137ec48cdd667b5909dd03be7f50d37da2f4abcd65a65db805156dd1079977f8447e888345c3ae14f695e20a50a010008c8a041f80c8d6c4654506da746a132a3308d62015a35bce5d511d64f9e9ad8e98937c24d2bcda1ed3799a2a11772be17cd1120383bc8a48b40f2a28c6c2504010093d4ec7f2782e083f1d9b69016d208c95ec899229e1f7f150bac0535b78dd01fdecf897cb444412670137d94476256d39e77f39ec9a0c4b44d2ed972356a610d0000000000000000000000';
        const borshified_real_9657674 = abiDecoder.decodeMethod(raw_9657674).params[0].value;
        expect(borshified_real_9657674).to.be.equal('0x'+borshify(block9657674).toString('hex'));

        await this.bridge.initWithBlock(borshify(block9657410));
        await this.bridge.blockHashes(9657410);

        await this.bridge.addLightClientBlock(borshify(block9657547));
        await this.bridge.blockHashes(9657547);

        const now = await time.latest();
        await timeIncreaseTo(now.add(time.duration.seconds(3600)));

        await this.bridge.addLightClientBlock(borshify(block9657674));
        await this.bridge.blockHashes(9657674);

        for (let i = 0; i < block9657674.approvals_after_next.length; i++) {
            if (block9657674.approvals_after_next[i]) {
                if (await this.bridge.checkBlockProducerSignatureInLastBlock(i)) {
                    console.log(`Signature ${i} is OK`);
                } else {
                    console.log(`Signature ${i} is NOT OK`);
                }
            }
        }
    });
});
