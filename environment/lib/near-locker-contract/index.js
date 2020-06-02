const Web3 = require('web3');
const BN = require('bn.js');
const {
    BorshContract,
    hexToBuffer,
    readerToHex,
} = require('../borsh');

const borshSchema = {
    bool: {
        kind: 'function',
        ser: (b) => Buffer.from(Web3.utils.hexToBytes(b ? '0x01' : '0x00')),
        deser: (z) => readerToHex(1)(z) === '0x01',
    },
    initInput: {
        kind: 'struct',
        fields: [
            ['prover_account', 'string'],
            ['skip_client_call', 'bool'],
        ],
    },
    unlockTokenInput: {
        kind: 'struct',
        fields: [
            ['token_account', 'string'],
            ['new_owner_id', 'string'],
            ['amount', 'u128'],
            ['proof', 'Proof'],
        ],
    },
    Proof: {
        kind: 'struct',
        fields: [
            ['log_index', 'u64'],
            ['log_entry_data', ['u8']],
            ['receipt_index', 'u64'],
            ['receipt_data', ['u8']],
            ['header_data', ['u8']],
            ['proof', [
                ['u8'],
            ]],
        ],
    },
};

class TokenLockerContract extends BorshContract {
    constructor (account) {
        super(borshSchema, account, {
            viewMethods: [{
                methodName: 'initialized',
                inputFieldType: null,
                outputFieldType: 'bool',
            }],
            changeMethods: [{
                methodName: 'init',
                inputFieldType: 'initInput',
                outputFieldType: null,
            },
            {
                methodName: 'unlock_token',
                inputFieldType: 'unlockTokenInput',
                outputFieldType: null,
            },
            ],
        });
    }

    // Call initialization methods on the contract.
    // If `skip_client_call` is true will not verify the PoW by calling the
    // client.
    async maybeInitialize (prover_account, skip_client_call) {
        await this.accessKeyInit();
        let initialized = false;
        try {
            initialized = await this.initialized();
        } catch (e) {
            // I guess not
        }
        if (!initialized) {
            console.log('Initializing token locker');
            await this.init({
                prover_account: prover_account,
                skip_client_call: skip_client_call,
            },
            new BN('1000000000000000'));
            console.log('Token locker initialized');
        }
    }
}

exports.TokenLockerContract = TokenLockerContract;
