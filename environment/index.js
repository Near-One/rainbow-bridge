const path = require('path');
const { program } = require('commander');

const { CleanCommand } = require('./commands/clean');
const { PrepareCommand } = require('./commands/prepare');
const { StartEthRelayCommand } = require('./commands/start/eth-relay.js');
const { StartNearRelayCommand } = require('./commands/start/near-relay.js');
const { StartGanacheNodeCommand } = require('./commands/start/ganache.js');
const { StartLocalNearNodeCommand } = require('./commands/start/near.js');
const { StopLocalNearNodeCommand } = require('./commands/stop/near.js');
const { StopManagedProcessCommand } = require('./commands/stop/process.js');
const { TransferETHERC20ToNear } = require('./commands/transfer-eth-erc20-to-near');
const { InitETHLocker } = require('./commands/init-eth-locker');
const { InitETHERC20 } = require('./commands/init-eth-erc20');
const { InitNEARContracts } = require('./commands/init-near-contracts');
const { InitNEARFunToken } = require('./commands/init-near-fun-token');
const { ETHDump } = require('./commands/eth-dump');
const { RainbowConfig } = require('./lib/config');
const { InitEthEd25519 } = require('./commands/init-eth-ed25519');
const { InitNear2EthClient } = require('./commands/init-near2eth-client');

RainbowConfig.declareOption(
    'near-network-id',
    'The identifier of the NEAR network that the given NEAR node is expected to represent.',
);
RainbowConfig.declareOption(
    'near-node-url',
    'The URL of the NEAR node.',
);
RainbowConfig.declareOption(
    'eth-node-url',
    'The URL of the Ethereum node.',
);
RainbowConfig.declareOption(
    'near-master-account',
    'The account of the master account on NEAR blockchain that can be used to deploy and initialize the test contracts.' +
    ' This account will also own the initial supply of the fungible tokens.',
);
RainbowConfig.declareOption(
    'near-master-sk',
    'The secret key of the master account on NEAR blockchain.',
);
RainbowConfig.declareOption(
    'eth-master-sk',
    'The secret key of the master account on Ethereum blockchain.',
);
RainbowConfig.declareOption(
    'eth2near-client-account',
    'The account of the Eth2NearClient contract that can be used to accept ETH headers.',
    'eth2nearclient',
);
RainbowConfig.declareOption(
    'eth2near-client-sk',
    'The secret key of the Eth2NearClient account. If not specified will use master SK.',
);
RainbowConfig.declareOption(
    'eth2near-client-contract-path',
    'The path to the Wasm file containing the Eth2NearClient contract.',
    path.join(process.env.HOME, '.rainbowup/bridge/libs-rs/res/eth_client.wasm')
);
RainbowConfig.declareOption(
    'eth2near-client-init-balance',
    'The initial balance of Eth2NearClient contract in femtoNEAR.',
    '100000000000000000000000000',
);
RainbowConfig.declareOption(
    'eth2near-client-validate-ethash',
    'The initial balance of Eth2NearClient contract in femtoNEAR.',
    'true',
);
RainbowConfig.declareOption(
    'eth2near-prover-account',
    'The account of the Eth2NearProver contract that can be used to accept ETH headers.',
    'eth2nearprover',
);
RainbowConfig.declareOption(
    'eth2near-prover-sk',
    'The secret key of the Eth2NearProver account. If not specified will use master SK.',
);
RainbowConfig.declareOption(
    'eth2near-prover-contract-path',
    'The path to the Wasm file containing the Eth2NearProver contract.',
    path.join(process.env.HOME, '.rainbowup/bridge/libs-rs/res/eth_prover.wasm')
);
RainbowConfig.declareOption(
    'eth2near-prover-init-balance',
    'The initial balance of Eth2NearProver contract in femtoNEAR.',
    '100000000000000000000000000',
);
RainbowConfig.declareOption('daemon', 'Whether the process should be launched as a daemon.', 'true', true);
RainbowConfig.declareOption('bridge-src', 'Path to the rainbow-bridge source. It will be downloaded if not provided.', '');
RainbowConfig.declareOption('core-src', 'Path to the nearcore source. It will be downloaded if not provided.', '');
RainbowConfig.declareOption('nearup-src', 'Path to the nearup source. It will be downloaded if not provided.', '');

// User-specific arguments.
RainbowConfig.declareOption(
    'near-fun-token-account',
    'The account of the fungible token contract that will be used to mint tokens locked on Ethereum.',
    'nearfuntoken',
);
RainbowConfig.declareOption(
    'near-fun-token-sk',
    'The secret key of the fungible token account. If not specified will use master SK.',
);
RainbowConfig.declareOption(
    'near-fun-token-contract-path',
    'The path to the Wasm file containing the fungible contract. Note, this version of fungible contract should support minting.',
    path.join(process.env.HOME, '.rainbowup/bridge/libs-rs/res/fungible_token.wasm')
);
RainbowConfig.declareOption(
    'near-fun-token-init-balance',
    'The initial balance of fungible token contract in femtoNEAR.',
    '100000000000000000000000000',
);
RainbowConfig.declareOption(
    'eth-locker-address',
    'ETH address of the locker contract.',
);
RainbowConfig.declareOption(
    'eth-locker-abi-path',
    'Path to the .abi file defining Ethereum locker contract. This contract works in pair with mintable fungible token on NEAR blockchain.',
    path.join(process.env.HOME, '.rainbowup/bridge/libs-sol/token-locker/dist/TokenLocker.full.abi')
);
RainbowConfig.declareOption(
    'eth-locker-bin-path',
    'Path to the .bin file defining Ethereum locker contract. This contract works in pair with mintable fungible token on NEAR blockchain.',
    path.join(process.env.HOME, '.rainbowup/bridge/libs-sol/token-locker/dist/TokenLocker.full.bin')
);
RainbowConfig.declareOption(
    'eth-erc20-address',
    'ETH address of the ERC20 contract.',
);
RainbowConfig.declareOption(
    'eth-erc20-abi-path',
    'Path to the .abi file defining Ethereum ERC20 contract.',
    path.join(process.env.HOME, '.rainbowup/bridge/libs-sol/token-locker/dist/MyERC20.full.abi')
);
RainbowConfig.declareOption(
    'eth-erc20-bin-path',
    'Path to the .bin file defining Ethereum ERC20 contract.',
    path.join(process.env.HOME, '.rainbowup/bridge/libs-sol/token-locker/dist/MyERC20.full.bin')
);
RainbowConfig.declareOption(
    'eth-ed25519-address',
    'ETH address of the ED25519 contract.',
);
RainbowConfig.declareOption(
    'eth-ed25519-abi-path',
    'Path to the .abi file defining Ethereum ED25519 contract.',
    path.join(process.env.HOME, '.rainbowup/bridge/libs-sol/nearbridge/dist/Ed25519.full.abi')
);
RainbowConfig.declareOption(
    'eth-ed25519-bin-path',
    'Path to the .bin file defining Ethereum ED25519 contract.',
    path.join(process.env.HOME, '.rainbowup/bridge/libs-sol/nearbridge/dist/Ed25519.full.bin')
);
RainbowConfig.declareOption(
    'near2eth-client-address',
    'ETH address of the Near2EthClient contract.',
);
RainbowConfig.declareOption(
    'near2eth-client-abi-path',
    'Path to the .abi file defining Ethereum Near2EthClient contract.',
    path.join(process.env.HOME, '.rainbowup/bridge/libs-sol/nearbridge/dist/NearBridge.full.abi')
);
RainbowConfig.declareOption(
    'near2eth-client-bin-path',
    'Path to the .bin file defining Ethereum Near2EthClient contract.',
    path.join(process.env.HOME, '.rainbowup/bridge/libs-sol/nearbridge/dist/NearBridge.full.bin')
);
RainbowConfig.declareOption(
    'near2eth-prover-address',
    'ETH address of the Near2EthProver contract.',
);
RainbowConfig.declareOption(
    'near2eth-prover-abi-path',
    'Path to the .abi file defining Ethereum Near2EthProver contract.',
    path.join(process.env.HOME, '.rainbowup/bridge/libs-sol/nearprover/dist/NearProver.full.abi')
);
RainbowConfig.declareOption(
    'near2eth-prover-bin-path',
    'Path to the .bin file defining Ethereum Near2EthProver contract.',
    path.join(process.env.HOME, '.rainbowup/bridge/libs-sol/nearprover/dist/NearProver.full.bin')
);

program.version('0.1.0');

// General-purpose commands.
program.command('clean').action(CleanCommand.execute);

RainbowConfig.addOptions(
    program.command('prepare')
        .action(PrepareCommand.execute),
    [
        'bridge-src',
        'core-src',
        'nearup-src',
    ]);

// Maintainer commands.

const startCommand = program.command('start');

startCommand.command('near-node')
    .action(StartLocalNearNodeCommand.execute);

RainbowConfig.addOptions(
    startCommand.command('ganache')
        .action(StartGanacheNodeCommand.execute),
    ['daemon'],
);

RainbowConfig.addOptions(
    startCommand.command('eth-relay')
        .action(StartEthRelayCommand.execute),
    [
        'near-master-account',
        'near-master-sk',
        'eth2near-client-account',
        'near-network-id',
        'near-node-url',
        'daemon',
    ],
);

RainbowConfig.addOptions(
    startCommand.command('near-relay')
        .action(StartNearRelayCommand.execute),
    [
        'eth-node-url',
        'eth-master-sk',
        'near-node-url',
        'near-network-id',
        'near2eth-client-abi-path',
        'near2eth-client-address',
        'daemon',
    ],
);

const stopCommand = program.command('stop');

stopCommand.command('near-node')
    .action(StopLocalNearNodeCommand.execute);

stopCommand.command('ganache')
    .action(StopManagedProcessCommand.execute);

stopCommand.command('eth-relay')
    .action(StopManagedProcessCommand.execute);

RainbowConfig.addOptions(
    program.command('init-near-contracts')
        .description('Deploys and initializes Eth2NearClient and Eth2NearProver contracts to NEAR blockchain.')
        .action(InitNEARContracts.execute),
    [
        'near-network-id',
        'near-node-url',
        'eth-node-url',
        'near-master-account',
        'near-master-sk',
        'eth2near-client-account',
        'eth2near-client-sk',
        'eth2near-client-contract-path',
        'eth2near-client-init-balance',
        'eth2near-client-validate-ethash',
        'eth2near-prover-account',
        'eth2near-prover-sk',
        'eth2near-prover-contract-path',
        'eth2near-prover-init-balance',
    ]);

RainbowConfig.addOptions(
    program.command('init-eth-ed25519')
        .description('Deploys and initializes ED25519 Solidity contract. It replaces missing precompile.')
        .action(InitEthEd25519.execute),
    [
        'eth-node-url',
        'eth-master-sk',
        'eth-ed25519-abi-path',
        'eth-ed25519-bin-path',
    ]);

RainbowConfig.addOptions(
    program.command('init-near2eth-client')
        .description('Deploys and initializes Near2EthClient.')
        .action(InitNear2EthClient.execute),
    [
        'eth-node-url',
        'eth-master-sk',
        'near2eth-client-abi-path',
        'near2eth-client-bin-path',
        'eth-ed25519-address',
    ]);

// User commands.

RainbowConfig.addOptions(
    program.command('init-near-fun-token')
        .description('Deploys and initializes mintable fungible token to NEAR blockchain. Requires locker on Ethereum side.')
        .action(InitNEARFunToken.execute),
    [
        'near-fun-token-account',
        'near-fun-token-sk',
        'near-fun-token-contract-path',
        'near-fun-token-init-balance',
    ],
);

RainbowConfig.addOptions(
    program.command('init-eth-locker')
        .description('Deploys and initializes locker contract on Ethereum blockchain. Requires mintable fungible token on Near side.')
        .action(InitETHLocker.execute),
    [
        'eth-node-url',
        'eth-master-sk',
        'eth-locker-abi-path',
        'eth-locker-bin-path',
    ],
);

RainbowConfig.addOptions(
    program.command('init-eth-erc20')
        .description('Deploys and initializes ERC20 contract on Ethereum blockchain.')
        .action(InitETHERC20.execute),
    [
        'eth-node-url',
        'eth-master-sk',
        'eth-erc20-abi-path',
        'eth-erc20-bin-path',
    ],
);

RainbowConfig.addOptions(
    program.command('transfer-eth-erc20-to-near')
        .action(TransferETHERC20ToNear.execute)
        .option('--amount <amount>', 'Amount of ERC20 tokens to transfer')
        .option('--eth-sender-sk <eth_sender_sk>', 'The secret key of the Ethereum account that will be sending ERC20 token.')
        .option('--near-receiver-account <near_receiver_account>', 'The account on NEAR blockchain that will be receiving the minted token.'),
    [
        'eth-node-url',
        'eth-erc20-address',
        'eth-erc20-abi-path',
        'eth-locker-address',
        'eth-locker-abi-path',
        'near-node-url',
        'near-network-id',
        'near-fun-token-account',
        'eth2near-client-account',
        'near-master-account',
        'near-master-sk',
    ],
);

program.command('eth-dump <kind_of_data>')
    .option('--eth-node-url <eth_node_url>', 'ETH node API url')
    .option('--path <path>', 'Dir path to dump eth headers')
    .option('--start-block <start_block>', 'Start block number (inclusive), default to be 4.3K blocks away from start block')
    .option('--end-block <end_block>', 'End block number (inclusive), default to be latest block')
    .action(ETHDump.execute);

(async () => { await program.parseAsync(process.argv); })();
