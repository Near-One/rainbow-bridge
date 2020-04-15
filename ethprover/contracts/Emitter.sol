pragma solidity ^0.5.0;


contract Emitter {
    event Log(uint256 indexed a, uint256 b, uint256 c);

    function emitEvent(uint256 a, uint256 b, uint256 c) public {
        emit Log(a, b, c);
    }
}
