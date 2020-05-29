'use strict'

const {StartCommand} = require('./commands/start');
const {program} = require('commander');

program.version('0.1.0');

async function main() {
  program.command('start <service>').action(StartCommand.execute);
  await program.parseAsync(process.argv);
}

main();
