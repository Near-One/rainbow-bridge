// Copyright (c) 2019 Richard Moore
// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the Software without restriction,
// including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so,
// subject to the following conditions:
// The above copyright notice and this permission notice shall be included in all copies or substantial portions
// of the Software.
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE
// WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS
// OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
// TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

// Note: this code copied and edited from 'ethers' project due to missing EIP1559 support.

const ethers = require('ethers');
const hwAppEth = require('@ledgerhq/hw-app-eth').default;
const hid = require('@ledgerhq/hw-transport-node-hid').default;
const defaultPath = "m/44'/60'/0'/0/0";

function waiter (duration) {
    return new Promise((resolve) => {
        setTimeout(resolve, duration);
    });
}

class EthLedgerSigner extends ethers.Signer {
    constructor (provider, path) {
        super();
        if (!path) {
            path = defaultPath;
            console.log('Using default HD path');
        }
        console.log(`Using Ledger key path: ${path}`);
        ethers.utils.defineReadOnly(this, 'path', path);
        ethers.utils.defineReadOnly(this, 'provider', provider || null);
        ethers.utils.defineReadOnly(this, '_eth', hid.create().then((transport) => {
            const eth = new hwAppEth(transport);
            return eth.getAppConfiguration().then((_config) => {
                return eth;
            }, (error) => {
                return Promise.reject(error);
            });
        }, (error) => {
            return Promise.reject(error);
        }));
    }

    _retry (callback, timeout) {
        return new Promise(async (resolve, reject) => {
            if (timeout && timeout > 0) {
                setTimeout(() => {
                    reject(new Error('timeout'));
                }, timeout);
            }
            const eth = await this._eth;
            // Wait up to 5 seconds
            for (let i = 0; i < 50; i++) {
                try {
                    const result = await callback(eth);
                    return resolve(result);
                } catch (error) {
                    if (error.id !== 'TransportLocked') {
                        return reject(error);
                    }
                }
                await waiter(100);
            }
            return reject(new Error('timeout'));
        });
    }

    async getAddress () {
        const account = await this._retry((eth) => eth.getAddress(this.path));
        return ethers.utils.getAddress(account.address);
    }

    async signMessage (message) {
        if (typeof message === 'string') {
            message = ethers.utils.toUtf8Bytes(message);
        }
        const messageHex = ethers.utils.hexlify(message).substring(2);
        const sig = await this._retry((eth) => eth.signPersonalMessage(this.path, messageHex));
        sig.r = '0x' + sig.r;
        sig.s = '0x' + sig.s;
        return ethers.utils.joinSignature(sig);
    }

    async signTransaction (transaction) {
        const tx = await ethers.utils.resolveProperties(transaction);
        const baseTx = {
            chainId: tx.chainId || undefined,
            data: tx.data || undefined,
            gasLimit: tx.gasLimit || undefined,
            gasPrice: tx.gasPrice || undefined,
            nonce: tx.nonce ? ethers.BigNumber.from(tx.nonce).toNumber() : undefined,
            maxFeePerGas: tx.maxFeePerGas || undefined, // EIP1559 https://github.com/ethers-io/ethers.js/pull/2056
            maxPriorityFeePerGas: tx.maxPriorityFeePerGas || undefined, // EIP1559 https://github.com/ethers-io/ethers.js/pull/2056
            type: tx.type, // EIP1559 https://github.com/ethers-io/ethers.js/pull/2056
            to: tx.to || undefined,
            value: tx.value || undefined,
        };
        const unsignedTx = ethers.utils.serializeTransaction(baseTx).substring(2);
        console.log('Using public key:', await this.getAddress());
        console.log('Waiting for confirmation on Ledger...');
        const sig = await this._retry((eth) => eth.signTransaction(this.path, unsignedTx));
        return ethers.utils.serializeTransaction(baseTx, {
            v: ethers.BigNumber.from('0x' + sig.v).toNumber(),
            r: '0x' + sig.r,
            s: '0x' + sig.s,
        });
    }

    connect (provider) {
        return new EthLedgerSigner(provider, this.path);
    }
}
exports.EthLedgerSigner = EthLedgerSigner;
