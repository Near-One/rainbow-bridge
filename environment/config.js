const Configstore = require('configstore');
const path = require('path');
const homedir = require('os').homedir();

const rainbowDir = path.join(homedir, '.rainbowup', 'config.json');
const RainbowConfig = new Configstore('', {}, {
    configPath: rainbowDir,
});

exports.RainbowConfig = RainbowConfig;
