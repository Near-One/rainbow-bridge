
const { program } = require('commander');

const { CleanCommand } = require('./commands/clean');
const { PrepareCommand } = require('./commands/prepare');
const { StartEthRelayCommand } = require('./commands/start/eth-relay.js');
const { StartGanacheNodeCommand } = require('./commands/start/ganache.js');
const { StartLocalNearNodeCommand } = require('./commands/start/near.js');
const { TestCommand } = require('./commands/test');
const { TransferFunETH2NEAR } = require('./commands/transfer-fun-eth2near');
const { InitETHTestContracts } = require('./commands/init-eth-test-contracts');
const { InitNEARTestContracts } = require('./commands/init-near-test-contracts');
const { ETHDump } = require('./commands/eth-dump');

program.version('0.1.0');

program.command('clean').action(CleanCommand.execute);

const startCommand = program.command('start');

startCommand.command('near-node')
    .action(StartLocalNearNodeCommand.execute);

startCommand.command('ganache')
    .action(StartGanacheNodeCommand.execute);

startCommand.command('eth-relay')
    .action(StartEthRelayCommand.execute)
    .option(
        '--near-master-account <near_master_account>',
        'The account of the master account on NEAR that can be used to deploy and initialize the test contracts.' +
            ' This account will also own the initial supply of the fungible tokens.',
        '',
    )
    .option(
        '--near-master-sk <near_master_sk>',
        'The secret key of the master account.',
    )
    .option(
        '--near-network-id <near_network_id>',
        'The identifier of the NEAR network that the given NEAR node is expected to represent.',
        'local',
    )
    .option(
        '--near-node-url <near_node_url>',
        'The URL of the NEAR node.',
        '',
    )
    .option(
        '--eth-node-url <eth_node_url>',
        'The URL of the Ethereum node.',
        '',
    );

program.command('prepare')
    .action(PrepareCommand.execute)
    .option('--bridge-src <bridge_src>', 'Path to the rainbow-bridge source',
        '')
    .option('--core-src <core_src>', 'Path to the nearcore source', '')
    .option('--nearup-src <nearup_src>', 'Path to the nearup source', '')
;

program.command('test').action(TestCommand.execute);

program.command('transfer-fun-eth2near')
    .action(TransferFunETH2NEAR.execute)
    .option('--eth-node-url <eth_node_url>', 'The URL of the Ethereum node.',
        '')
    .option(
        '--eth-token-address <eth_token_address>',
        'Address of the ERC20 token on Ethereum network that will be transferred.',
        '')
    .option('--eth-token-abi-path <eth_token_abi_path>',
        'Path to an ABI file describing Ethereum token contract interface',
        '')
    .option(
        '--eth-locker-address <eth_locker_address>',
        'Address of the token locker contract on Ethereum network that will be locking the token on Ethereum side.',
        '')
    .option('--eth-locker-abi-path <eth_locker_abi_path>',
        'Path to an ABI file describing Ethereum locker contract interface',
        '')
    .option('--eth-sender-sk <eth_sender_sk>',
        'Secret key of the account that owns the token.', '')
    .option('--near-node-url <near_node_url>', 'The URL of the NEAR node.', '')
    .option(
        '--near-network-id <near_network_id>',
        'The identifier of the NEAR network that the given NEAR node is expected to represent.',
        '')
    .option(
        '--near-token-address <near_token_address>',
        'Address of the fungible token on NEAR network that will be transferred.',
        '')
    .option(
        '--near-receiver-account <near_receiver_account>',
        'Address of the account that will be receiving the token on NEAR side. This account will also be paying for the gas.',
        '')
    .option(
        '--near-receiver-sk <near_receier_sk>',
        'Secret key of the account on NEAR that will be receiving the token. This key will be used to pay for the gas.',
        '')
    .option('--amount <amount>', 'Amount of tokens to transfer.', '');

program.command('init-eth-test-contracts')
    .action(InitETHTestContracts.execute)
    .option('--eth-node-url <eth_node_url>', 'The URL of the Ethereum node.',
        '')
    .option(
        '--eth-master-sk <eth_master_sk>',
        'The secret key of the master account on Ethereum that can be used to deploy and initialize the test contracts.' +
            ' This account will also own the initial supply of the ERC20 tokens.',
        '')
    .option('--contracts-dir <contracts_dir>',
        'The path to the abi and bin files of the test contracts.', '');

program.command('init-near-test-contracts')
    .action(InitNEARTestContracts.execute)
    .option('--near-node-url <near_node_url>', 'The URL of the NEAR node.', '')
    .option(
        '--near-network-id <near_network_id>',
        'The identifier of the NEAR network that the given NEAR node is expected to represent.',
        '')
    .option('--contracts-dir <contracts_dir>',
        'The path to the wasm files of the test contracts.', '')
    .option(
        '--near-master-account <near_master_account>',
        'The account of the master account on NEAR that can be used to deploy and initialize the test contracts.' +
            ' This account will also own the initial supply of the fungible tokens.',
        '')
    .option('--near-master-sk <near_master_sk>',
        'The secret key of the master account.', '')
    .option('--validate_ethash <validate_ethash>', '', 'true')
    .option(
        '--near-prover-account <near_prover_account>',
        'The account of the prover contract that it locker contract can use to validate proofs.',
        '');

program.command('eth-dump <kind_of_data>')
    .option('--eth-node-url <eth_node_url>', 'ETH node API url')
    .option('--path <path>', 'Dir path to dump eth headers')
    .option('--start-block <start_block>', 'Start block number (inclusive), default to be 4.3K blocks away from start block')
    .option('--end-block <end_block>', 'End block number (inclusive), default to be latest block')
    .action(ETHDump.execute);
    
(async () => { await program.parseAsync(process.argv); })();
