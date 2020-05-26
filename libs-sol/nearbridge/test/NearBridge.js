const bs58 = require('bs58')

const NearBridge = artifacts.require('NearBridge');
const NearDecoder = artifacts.require('NearDecoder');

function cookLightClientBlock(block) {
    
    console.log([
        bs58.decode(block.prev_block_hash),
        bs58.decode(block.next_block_inner_hash),
        
        web3.utils.toBN(block.inner_lite.height).toBuffer('le', 8),
        bs58.decode(block.inner_lite.epoch_id),
        bs58.decode(block.inner_lite.next_epoch_id),
        bs58.decode(block.inner_lite.prev_state_root),
        bs58.decode(block.inner_lite.outcome_root),
        web3.utils.toBN(block.inner_lite.timestamp).toBuffer('le', 8),
        bs58.decode(block.inner_lite.next_bp_hash),
        bs58.decode(block.inner_lite.block_merkle_root),

        bs58.decode(block.inner_rest_hash),

        Buffer.from([1]),
        web3.utils.toBN(block.next_bps.length).toBuffer('le', 4),
        block.next_bps.map(next_bp => [
            web3.utils.toBN(next_bp.account_id.length).toBuffer('le', 4),
            Buffer.from(next_bp.account_id),
            next_bp.public_key.substr(0, 8) === 'ed25519:' ? Buffer.from([0]) : Buffer.from([1]),
            bs58.decode(next_bp.public_key.substr(8)),
            web3.utils.toBN(next_bp.stake).toBuffer('le', 16)
        ]),

        web3.utils.toBN(block.approvals_after_next.length).toBuffer('le', 4),
        block.approvals_after_next.map(
            signature => Buffer.concat([
                Buffer.from([signature ? 1 : 0]),
                signature ? bs58.decode(signature.substr(8)) : Buffer.from([])
            ])
        ),
    ]);
    
    return Buffer.concat([
        bs58.decode(block.prev_block_hash),
        bs58.decode(block.next_block_inner_hash),
        Buffer.concat([
            web3.utils.toBN(block.inner_lite.height).toBuffer('le', 8),
            bs58.decode(block.inner_lite.epoch_id),
            bs58.decode(block.inner_lite.next_epoch_id),
            bs58.decode(block.inner_lite.prev_state_root),
            bs58.decode(block.inner_lite.outcome_root),
            web3.utils.toBN(block.inner_lite.timestamp).toBuffer('le', 8),
            bs58.decode(block.inner_lite.next_bp_hash),
            bs58.decode(block.inner_lite.block_merkle_root),
        ]),
        bs58.decode(block.inner_rest_hash),

        Buffer.from([1]),
        web3.utils.toBN(block.next_bps.length).toBuffer('le', 4),
        Buffer.concat(
            block.next_bps.map(next_bp => Buffer.concat([
                web3.utils.toBN(next_bp.account_id.length).toBuffer('le', 4),
                Buffer.from(next_bp.account_id),
                next_bp.public_key.substr(0, 8) === 'ed25519:' ? Buffer.from([0]) : Buffer.from([1]),
                bs58.decode(next_bp.public_key.substr(8)),
                web3.utils.toBN(next_bp.stake).toBuffer('le', 16)
            ])),
        ),

        web3.utils.toBN(block.approvals_after_next.length).toBuffer('le', 4),
        Buffer.concat(
            block.approvals_after_next.map(
                signature => Buffer.concat([
                    Buffer.from([signature ? 1 : 0]),
                    signature ? bs58.decode(signature.substr(8)) : Buffer.from([])
                ])
            ),
        ),
    ]);
}

contract('NearBridge', function ([_, addr1]) {
    beforeEach(async function () {
        this.decoder = await NearDecoder.new();
        this.bridge = await NearBridge.new(
            "0xedb63664f3b62c4a24ab7acf1c4462ad55217748814fe6aea9bc0453694635b7",
            "0x81039bbb1b93afa4d586b867ac068bc7170421b01a6a802e4b2e29e5e8357bf8"
        );
    });

    it('should be ok', async function () {
        const data = cookLightClientBlock(require('./block_1498.json'));
        console.log(data.toString('hex'));
        await this.bridge.deposit({ value: web3.utils.toWei('1') });
        await this.bridge.addLightClientBlock(data);
    });
});
