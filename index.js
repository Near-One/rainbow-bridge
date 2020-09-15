const nearAPI = require("near-api-js")

const getConfig = require("./src/get-config");
const utils = require('./src/utils');

// Re-export to make sure subsequent libraries use the same version.
// Without this, ppl would use different version of PublicKey class and the tranactions won't serialize. 
exports.nearlib = nearAPI;
exports.nearAPI = nearAPI;
exports.getConfig = getConfig;
exports.utils = utils;
