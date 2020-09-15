const Configstore = require('configstore')
const path = require('path')
const homedir = require('os').homedir()
const changeCase = require('change-case')

class RainbowConfig {
  // Remembers the description of the parameter.
  static declareOption(name, description, defaultValue = '', noConfig = false) {
    this.paramDeclarations[name] = {
      description: description,
      defaultValue: defaultValue,
      noConfig: noConfig,
    }
  }

  // Adds a list of options to the given commander action.
  // options is a list of string representing parameter name.
  static addOptions(action, options) {
    let prev = action
    for (const option of options) {
      const declaration = this.paramDeclarations[option]
      const paramCase = changeCase.paramCase(option)
      const snakeCase = changeCase.snakeCase(option)
      if (declaration.defaultValue) {
        prev = prev.option(
          `--${paramCase} <${snakeCase}>`,
          declaration.description,
          (value, previous) => {
            this._processArg(option, value, previous)
          }
        )
      } else {
        prev = prev.option(
          `--${paramCase} <${snakeCase}>`,
          declaration.description,
          (value, previous) => {
            this._processArg(option, value, previous)
          },
          declaration.defaultValue
        )
      }
    }
  }

  // This function is called when argument is processed by the commander.
  static _processArg(name, value, previous) {
    const constantCase = changeCase.constantCase(name)
    const camelCase = changeCase.camelCase(name)
    if (this.paramValues[name]) {
      console.error(`Argument ${name} is specified more than once.`)
      process.exit(1)
    }
    if (process.env[constantCase]) {
      this.paramValues[name] = {
        value: process.env[constantCase],
        paramType: 'env',
      }
    } else if (value) {
      this.paramValues[name] = { value: value, paramType: 'arg' }
    } else if (this.configFile.has(camelCase)) {
      this.paramValues[name] = {
        value: this.configFile.get(camelCase),
        paramType: 'config',
      }
    } else if (previous) {
      this.paramValues[name] = { value: previous, paramType: 'default' }
    } else {
      console.error('Unreachable code')
      process.exit(1)
    }
  }

  // This function should be used to retrieve the actual value of the argument.
  static getParam(name) {
    const res = this.maybeGetParam(name)
    if (res === null) {
      console.error(`Parameter ${name} must be specified.`)
      throw new Error('error')
      //process.exit(1)
    }
    return res
  }

  static maybeGetParam(name) {
    if (typeof this.paramValues[name] === 'undefined') {
      const camelCase = changeCase.camelCase(name)
      if (this.configFile.has(camelCase)) {
        return this.configFile.get(camelCase)
      } else {
        const decl = this.paramDeclarations[name]
        if (
          typeof decl !== 'undefined' &&
          typeof decl.defaultValue !== 'undefined'
        ) {
          return decl.defaultValue
        } else {
          return null
        }
      }
    } else {
      return this.paramValues[name].value
    }
  }

  static setParam(name, value) {
    this.paramValues[name] = { value: value, paramType: 'config' }
  }

  // Get all args, but without daemon as array of strings.
  static getArgsNoDaemon() {
    const result = []
    for (const name in this.paramValues) {
      const value = this.paramValues[name]
      if (value.paramType === 'arg' && name !== 'daemon') {
        result.push(`--${name}`)
        result.push(`${value.value}`)
      }
    }
    result.push('--daemon')
    result.push('false')
    return result
  }

  // Iterates over the params and writes them into config if they were set through arguments
  // or default values.
  static saveConfig() {
    for (const name in this.paramValues) {
      const value = this.paramValues[name]
      if (
        typeof this.paramDeclarations[name] === 'undefined' ||
        this.paramDeclarations[name].noConfig
      ) {
        continue
      }
      if (
        value.paramType === 'arg' ||
        value.paramType === 'default' ||
        value.paramType === 'config'
      ) {
        const camelCase = changeCase.camelCase(name)
        this.configFile.set(camelCase, value.value)
      }
    }
  }
}

// Stores key values 'my-param-name' => { value: 'value', paramType: ...}
// where paramType is one of 'env', 'arg', 'config', 'default' representing where the
// value is coming from.
RainbowConfig.paramValues = {}

// Stores key values 'my-param-name' => { description: 'my description', defaultValue: 'default value', noConfig: false}
// where default value might not be provided.
// noConfig indicates whether the param should not be written into config.
RainbowConfig.paramDeclarations = {}

// File that stores config.
RainbowConfig.configFile = new Configstore(
  '',
  {},
  {
    configPath: path.join(homedir, '.rainbow', 'config.json'),
  }
)

exports.RainbowConfig = RainbowConfig
