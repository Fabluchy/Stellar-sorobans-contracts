// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title SecurityEvents
/// @notice Centralized definition of security‑related events for the contract suite.
/// @dev All contracts that need to emit audit‑trail events should inherit from this contract.
contract SecurityEvents {
    /// @dev Emitted when a governance proposal is executed.
    /// @param executor   Address that called executeProposal.
    /// @param proposalId Unique identifier of the proposal.
    /// @param action     Human‑readable description of the action performed.
    /// @param timestamp  Block timestamp of execution.
    event GovernanceActionExecuted(
        address indexed executor,
        bytes32 indexed proposalId,
        string action,
        uint256 timestamp
    );
}
