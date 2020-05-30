'use strict'

const {program} = require('commander');

const {CleanCommand} = require('./commands/clean');
const {PrepareCommand} = require('./commands/prepare');
const {StartCommand} = require('./commands/start');
const {TestCommand} = require('./commands/test');

program.version('0.1.0');

program.command('clean').action(CleanCommand.execute);
program.command('start <service>').action(StartCommand.execute);
program.command('prepare')
    .action(PrepareCommand.execute)
    .option('--bridge-src <bridge_src>', 'Path to the rainbow-bridge source',
            '')
    .option('--core-src <core_src>', 'Path to the nearcore source', '');
program.command('test').action(TestCommand.execute);

program.parse(process.argv);
