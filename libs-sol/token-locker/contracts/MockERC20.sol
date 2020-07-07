pragma solidity ^0.5.0;
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";


contract MockERC20 is ERC20 {
    constructor() public {
        _mint(msg.sender, 1000000000);
    }

    function mint(address beneficiary, uint256 amount) public {
        _mint(beneficiary, amount);
    }
}
