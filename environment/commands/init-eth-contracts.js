const { RainbowConfig } = require('../config');

class InitEthContractsCommand {
    static execute (command) {
    	RainbowConfig.clear();
    	RainbowConfig.set('nearNodeUrl', command.nearNodeUrl);
    	RainbowConfig.set('nearNetworkId', command.nearNetworkId);
    	RainbowConfig.set('ethNodeUrl', command.ethNodeUrl);

    	RainbowConfig.set('masterAccount', command.masterAccount);
    	RainbowConfig.set('masterSk', command.masterSk);

    	RainbowConfig.set('clientAccount', command.clientAccount);
    	RainbowConfig.set('clientSk', command.clientSk);
    	RainbowConfig.set('clientContractPath', command.clientContractPath);
    	RainbowConfig.set('clientInitBalance', command.clientInitBalance);

    	RainbowConfig.set('proverAccount', command.proverAccount);
    	RainbowConfig.set('proverSk', command.proverSk);
    	RainbowConfig.set('proverContractPath', command.proverContractPath);
    	RainbowConfig.set('proverInitBalance', command.proverInitBalance);

    	RainbowConfig.set('validateEthash', command.validateEthash);

    	console.log('The current Rainbow configuration is:');
    	console.log(RainbowConfig.all);
    }
}

exports.InitEthContractsCommand = InitEthContractsCommand;
