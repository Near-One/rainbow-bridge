pragma solidity ^0.5.0;
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/token/ERC20/SafeERC20.sol";


contract MyERC20 is ERC20 {
    constructor() public {
        _mint(msg.sender, 1000000000);
    }
}
