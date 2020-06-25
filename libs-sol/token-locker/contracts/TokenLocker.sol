pragma solidity ^0.5.0;
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/SafeERC20.sol";
contract TokenLocker {
    using SafeERC20 for IERC20;
    event Locked(
        address indexed token,
        address indexed sender,
        uint256 amount,
        string accountId
    );
    function lockToken(IERC20 token, uint256 amount, string memory accountId) public {
        token.safeTransferFrom(msg.sender, address(this), amount);
        emit Locked(address(token), msg.sender, amount, accountId);
    }
}
