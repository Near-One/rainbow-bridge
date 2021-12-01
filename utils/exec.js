const util = require('util')
const exec = util.promisify(require('child_process').exec)

async function execAsync (cmd) {
  const res = await exec(cmd)
  if (res.stderr) {
    throw new Error(`Error to exec the command """${cmd}""": ${res.stderr}`)
  }
  return JSON.parse(res.stdout)
}

exports.execAsync = execAsync
