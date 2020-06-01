const {RainbowRunner} = require('../../lib/runner');

class StartCommand {
  static async execute(service) { await (new RainbowRunner()).run(service) }
}

exports.StartCommand = StartCommand;
