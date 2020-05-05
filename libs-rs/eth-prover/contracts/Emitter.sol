pragma solidity ^0.5.0;


contract Emitter {
    event Log(uint256 indexed a, uint256 b, uint256 c);
    event Log2(uint256 indexed a, uint256 b, uint256 c);

    function emitEvent(uint256 a, uint256 b, uint256 c) public {
        emit Log(a, b, c);
    }

    function emitEvent2(uint256 a, uint256 b, uint256 c) public {
        emit Log2(a, b, c);
    }
}
