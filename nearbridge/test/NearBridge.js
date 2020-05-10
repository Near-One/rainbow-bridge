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

        bs58.decode(block.inner_rest_hash),

        Buffer.from([1]),
        web3.utils.toBN(block.next_bps.length).toBuffer('le', 4),
        block.next_bps.map(next_bp => [
            web3.utils.toBN(next_bp.account_id.length).toBuffer('le', 4),
            Buffer.from(next_bp.account_id),
            bs58.decode(next_bp.public_key.substr(8)),
            web3.utils.toBN(next_bp.stake).toBuffer('le', 16)
        ]),

        web3.utils.toBN(block.approvals_next.length).toBuffer('le', 4),
        block.approvals_next.map(
            approval => Buffer.concat([
                Buffer.from([approval ? 1 : 0]),
                approval ? bs58.decode(approval.signature.substr(8)) : Buffer.from([])
            ])
        ),

        web3.utils.toBN(block.approvals_after_next.length).toBuffer('le', 4),
        block.approvals_after_next.map(
            approval => Buffer.concat([
                Buffer.from([approval ? 1 : 0]),
                approval ? bs58.decode(approval.signature.substr(8)) : Buffer.from([])
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
        ]),
        bs58.decode(block.inner_rest_hash),

        Buffer.from([1]),
        web3.utils.toBN(block.next_bps.length).toBuffer('le', 4),
        Buffer.concat(
            block.next_bps.map(next_bp => Buffer.concat([
                web3.utils.toBN(next_bp.account_id.length).toBuffer('le', 4),
                Buffer.from(next_bp.account_id),
                bs58.decode(next_bp.public_key.substr(8)),
                web3.utils.toBN(next_bp.stake).toBuffer('le', 16)
            ])),
        ),

        web3.utils.toBN(block.approvals_next.length).toBuffer('le', 4),
        Buffer.concat(
            block.approvals_next.map(
                approval => Buffer.concat([
                    Buffer.from([approval ? 1 : 0]),
                    approval ? bs58.decode(approval.signature.substr(8)) : Buffer.from([])
                ])
            )
        ),

        web3.utils.toBN(block.approvals_after_next.length).toBuffer('le', 4),
        Buffer.concat(
            block.approvals_after_next.map(
                approval => Buffer.concat([
                    Buffer.from([approval ? 1 : 0]),
                    approval ? bs58.decode(approval.signature.substr(8)) : Buffer.from([])
                ])
            ),
        ),
    ]);
}

contract('NearBridge', function ([_, addr1]) {
    beforeEach(async function () {
        this.decoder = await NearDecoder.new();
        this.bridge = await NearBridge.new(
            "0xab3e68948022cb53ad0317d3502fef5968628f2f2f1426ac284ef4a4d6360cd8",
            "0x7a7b473b80e40288450dcb59bff8e0ae22b38f74361b5e65bc3a3e68ff00377e"
        );
    });

    it('should be ok', async function () {
        const data = cookLightClientBlock(require('./block_1736172.json'));
        console.log(data.toString('hex'));
        await this.bridge.addLightClientBlock(data);
    });
});
