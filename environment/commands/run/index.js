const {RainbowRunner} = require('../../lib/runner');

class RunCommand {
  async execute(service) { await (new RainbowRunner()).run(service) }
}

exports.RunCommand = RunCommand;
