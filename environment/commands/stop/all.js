const { StopLocalNearNodeCommand } = require('./near.js');
const { StopManagedProcessCommand } = require('./process.js');

// TODO please do it in a better way

class A {
}

stop = function(name) {
    try {
        var a = A;
        a._name = name;
        StopManagedProcessCommand.execute(a);
    } catch (err) {
        console.log('Error stopping', name, err);
    }
}

class StopAllCommands {
    static execute() {
        console.log('Stopping all processes...');

        StopLocalNearNodeCommand.execute();
        for (const name of ['ganache', 'eth-relay', 'near-relay', 'near-watchdog']) {
            stop(name);
        }

        console.log('Stopping all processes done');
    }
}

exports.StopAllCommands = StopAllCommands;
