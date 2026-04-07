//! Hook development helpers for the ERC-8183 Agentic Commerce Protocol.
//!
//! Provides hook action selector constants matching the `BaseACPHook` routing
//! logic. These selectors are the `msg.sig` values passed by the core contract
//! to `IACPHook.beforeAction` / `IACPHook.afterAction` callbacks.
//!
//! # Hook data encoding (per selector)
//!
//! | Action | `data` encoding |
//! |---|---|
//! | `setProvider` | `abi.encode(address provider, bytes optParams)` |
//! | `setBudget` | `abi.encode(uint256 amount, bytes optParams)` |
//! | `fund` | `optParams` (raw bytes) |
//! | `submit` | `abi.encode(bytes32 deliverable, bytes optParams)` |
//! | `complete` | `abi.encode(bytes32 reason, bytes optParams)` |
//! | `reject` | `abi.encode(bytes32 reason, bytes optParams)` |

use alloy::{primitives::FixedBytes, sol_types::SolCall};

use crate::contracts::IERC8183;

/// `setProvider(uint256,address,bytes)` selector.
pub const SET_PROVIDER: FixedBytes<4> = FixedBytes(IERC8183::setProviderCall::SELECTOR);

/// `setBudget(uint256,uint256,bytes)` selector.
pub const SET_BUDGET: FixedBytes<4> = FixedBytes(IERC8183::setBudgetCall::SELECTOR);

/// `fund(uint256,uint256,bytes)` selector.
pub const FUND: FixedBytes<4> = FixedBytes(IERC8183::fundCall::SELECTOR);

/// `submit(uint256,bytes32,bytes)` selector.
pub const SUBMIT: FixedBytes<4> = FixedBytes(IERC8183::submitCall::SELECTOR);

/// `complete(uint256,bytes32,bytes)` selector.
pub const COMPLETE: FixedBytes<4> = FixedBytes(IERC8183::completeCall::SELECTOR);

/// `reject(uint256,bytes32,bytes)` selector.
pub const REJECT: FixedBytes<4> = FixedBytes(IERC8183::rejectCall::SELECTOR);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selectors_are_unique() {
        let all = [SET_PROVIDER, SET_BUDGET, FUND, SUBMIT, COMPLETE, REJECT];
        for (i, a) in all.iter().enumerate() {
            for b in all.iter().skip(i + 1) {
                assert_ne!(a, b, "duplicate selector found: {a}");
            }
        }
    }

    #[test]
    fn selectors_are_nonzero() {
        let all = [SET_PROVIDER, SET_BUDGET, FUND, SUBMIT, COMPLETE, REJECT];
        for sel in &all {
            assert_ne!(*sel, FixedBytes::ZERO, "selector should not be zero");
        }
    }
}
