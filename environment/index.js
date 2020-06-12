
const { program } = require('commander');

const { CleanCommand } = require('./commands/clean');
const { PrepareCommand } = require('./commands/prepare');
const { StartEthRelayCommand } = require('./commands/start/eth-relay.js');
const { StartGanacheNodeCommand } = require('./commands/start/ganache.js');
const { StartLocalNearNodeCommand } = require('./commands/start/near.js');
const { StopLocalNearNodeCommand } = require('./commands/stop/near.js');
const { StopManagedProcessCommand } = require('./commands/stop/process.js');
const { TransferFunETH2NEAR } = require('./commands/transfer-fun-eth2near');
const { InitNEARContracts } = require('./commands/init-near-contracts');
const { InitETHTestContracts } = require('./commands/init-eth-test-contracts');
const { InitNEARTestContracts } = require('./commands/init-near-test-contracts');
const { ETHDump } = require('./commands/eth-dump');

program.version('0.1.0');

program.command('clean').action(CleanCommand.execute);

const startCommand = program.command('start');

startCommand.command('near-node')
    .action(StartLocalNearNodeCommand.execute);

startCommand.command('ganache')
    .action(StartGanacheNodeCommand.execute)
    .option('--daemon [daemon]', 'Whether the process should be launched as a daemon.', 'true')
;

startCommand.command('eth-relay')
    .action(StartEthRelayCommand.execute)
    .option(
        '--master-account <master_account>',
        'The account on NEAR that can be used to submit headers to the client.',
    )
    .option(
        '--master-sk <master_sk>',
        'The secret key of the master account.',
    )
    .option(
        '--client-account <client_account>',
        'The account of Eth2NearClient contract.',
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
    )
    .option('--daemon [daemon]', 'Whether the process should be launched as a daemon.', 'true')
;

const stopCommand = program.command('stop');

stopCommand.command('near-node')
    .action(StopLocalNearNodeCommand.execute);

stopCommand.command('ganache')
    .action(StopManagedProcessCommand.execute);

stopCommand.command('eth-relay')
    .action(StopManagedProcessCommand.execute);

program.command('prepare')
    .action(PrepareCommand.execute)
    .option('--bridge-src <bridge_src>', 'Path to the rainbow-bridge source',
        '')
    .option('--core-src <core_src>', 'Path to the nearcore source', '')
    .option('--nearup-src <nearup_src>', 'Path to the nearup source', '')
;

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

program.command('init-near-contracts')
    .description('deploys and initializes Eth2NearClient and Eth2NearProver contracts to NEAR blockchain.')
    .action(InitNEARContracts.execute)
    .option('--near-node-url <near_node_url>', 'The URL of the NEAR node.')
    .option(
        '--near-network-id <near_network_id>',
        'The identifier of the NEAR network that the given NEAR node is expected to represent.')
    .option(
        '--master-account <master_account>',
        'The account of the master account on NEAR blockchain that can be used to deploy and initialize the test contracts.' +
        ' This account will also own the initial supply of the fungible tokens.')
    .option('--master-sk <master_sk>',
        'The secret key of the master account on NEAR blockchain.')
    .option(
        '--client-account <client_account>',
        'The account of the Eth2NearClient contract that can be used to accept ETH headers.', 'eth2nearclient')
    .option('--client-sk [client_sk]',
        'The secret key of the Eth2NearClient account. If not specified will use master SK.')
    .option('--client-contract-path <client_contract_path>',
        'The path to the Wasm file containing the Eth2NearClient contract.')
    .option('--client-init-balance <client_init_balance>',
        'The initial balance of Eth2NearClient contract in femtoNEAR.', '100000000000000000000000000')
    .option('--validate-ethash [validate_ethash]', 'Whether Eth2NearClient contract needs to validate the PoW.' +
        ' Should only be set to false for testing and diagnostics.', 'true')
    .option(
        '--prover-account <prover_account>',
        'The account of the Eth2NearProver contract that can be used to validate proofs.', 'eth2nearprover')
    .option('--prover-sk [prover_sk]',
        'The secret key of the Eth2NearProver account. If not specified will use master SK.')
    .option('--prover-contract-path <prover_contract_path>',
        'The path to the Wasm file containing the Eth2NearProver contract.')
    .option('--prover-init-balance <prover_init_balance>',
        'The initial balance of Eth2NearProver contract in femtoNEAR.', '100000000000000000000000000')
;

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
    .option(
        '--master-account <master_account>',
        'The master account on NEAR that can be used to deploy and initialize the test contracts.')
    .option('--master-sk <master_sk>',
        'The secret key of the master account.')
    .option('--contracts-dir <contracts_dir>',
        'The path to the wasm files of the test contracts.', '')
    .option(
        '--token-account <token_account>',
        'The account of the token contract that can be used to validate proofs.', 'eth2neartoken')
    .option('--token-sk [token_sk]',
        'The secret key of the token account. If not specified will use master SK.')
    .option('--token-contract-path <token_contract_path>',
        'The path to the Wasm file containing the token contract.')
    .option('--token-init-balance <token_init_balance>',
        'The initial balance of token contract in femtoNEAR.', '100000000000000000000000000')
    .option(
        '--prover-account <prover_account>',
        'The account of the Eth2NearProver contract that test contracts will use to validate proofs.',
        'eth2nearprover')
;

program.command('eth-dump <kind_of_data>')
    .option('--eth-node-url <eth_node_url>', 'ETH node API url')
    .option('--path <path>', 'Dir path to dump eth headers')
    .option('--start-block <start_block>', 'Start block number (inclusive), default to be 4.3K blocks away from start block')
    .option('--end-block <end_block>', 'End block number (inclusive), default to be latest block')
    .action(ETHDump.execute);

(async () => { await program.parseAsync(process.argv); })();
