'use strict'

const {program} = require('commander');

const {CleanCommand} = require('./commands/clean');
const {PrepareCommand} = require('./commands/prepare');
const {StartCommand} = require('./commands/start');
const {TestCommand} = require('./commands/test');

program.version('0.1.0');

async function main() {
  program.command('clean').action(CleanCommand.execute);
  program.command('start <service>').action(StartCommand.execute);
  program.command('prepare').action(PrepareCommand.execute);
  program.command('test').action(TestCommand.execute);
  await program.parseAsync(process.argv);
}

main();
