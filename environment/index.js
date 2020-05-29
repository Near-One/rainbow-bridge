'use strict'

const {RunCommand} = require('./commands/run');

(async function() { (new RunCommand()).execute(process.argv[2]); })()
