// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title EnumArrayShowcase
 * @notice Demonstrates dynamic enum array allocation and assignment.
 */
contract EnumArrayShowcase {
    enum WorkflowState {
        Created,
        Active,
        Suspended,
        Closed
    }

    function statesForReview(bool includeSuspended)
        external
        pure
        returns (WorkflowState[] memory states)
    {
        uint256 size = includeSuspended ? 3 : 2;
        states = new WorkflowState[](size);
        states[0] = WorkflowState.Created;
        states[1] = WorkflowState.Active;
        if (includeSuspended) {
            states[2] = WorkflowState.Suspended;
        }
    }
}
