const {BorshContract, hexToBuffer, readerToHex} = require('../borsh');

const borshSchema = {
    'initInput': {kind: 'struct', fields: [
            ['bridge_smart_contract', 'string']
        ]},
    'assertEthbridgeHashInput': { kind: 'struct', fields: [
            ['block_number', 'u64'],
            ['expected_block_hash', 'H256'],
        ]},
    'H256': {kind: 'function', ser: hexToBuffer, deser: readerToHex(32) },
};

class EthProverContract extends BorshContract {
    constructor(account) {
        super(borshSchema, account, {
            viewMethods: [],

            changeMethods: [{
                methodName: "init",
                inputFieldType: "initInput",
                outputFieldType: null,
            }, {
                methodName: "assert_ethbridge_hash",
                inputFieldType: "assertEthbridgeHashInput",
                outputFieldType: 'bool',
            }],
        })
    }

    async maybeInitialize(ethClientAccId) {
        await this.accessKeyInit();

        try {
            await this.init({
                bridge_smart_contract: ethClientAccId,
            });
            console.log('Initialized!');
        } catch (e) {
            // I guess not
            console.log('Probably already initialized', e);
        }
    }
}

exports.EthProverContract = EthProverContract;
