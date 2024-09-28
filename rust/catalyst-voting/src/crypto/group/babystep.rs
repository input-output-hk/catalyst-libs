//! Implementation of baby steps giant step algorithm to solve the discrete logarithm over
//! for the Ristretto255 group.

use std::collections::HashMap;

use super::{GroupElement, Scalar};

/// Default balance value.
/// Make steps asymmetric, in order to better use caching of baby steps.
/// Balance of 2 means that baby steps are 2 time more than `sqrt(max_votes)`
const DEFAULT_BALANCE: u64 = 2;

/// Holds precomputed baby steps for the baby-stap giant-step algorithm
/// for solving discrete log
#[derive(Debug, Clone)]
pub struct BabyStepGiantStep {
    /// Table of baby step precomputed values
    table: HashMap<GroupElement, u64>,
    /// baby step size value
    baby_step_size: u64,
    /// giant step value
    giant_step: GroupElement,
}

#[derive(thiserror::Error, Debug)]
pub enum BabyStepError {
    /// Invalid max value or balance
    #[error("Maximum value and balance must be greater than zero, provided max value: {0} and balance: {1}.")]
    InvalidMaxValueOrBalance(u64, u64),
    /// Max value exceeded
    #[error("Max log value exceeded. Means that the actual discret log for the provided group element is higher than the provided `max_log_value`.")]
    MaxLogExceeded,
}

impl BabyStepGiantStep {
    /// Creates a new setup for the baby-stap giant-step algorithm.
    ///
    /// Balance is used to make steps asymmetrical. If the table is reused multiple times
    /// with the same `max_value` it is recommended to set a balance > 1, since this
    /// will allow to cache more results, at the expense of a higher memory footprint.
    ///
    /// If not provided it will default to 2, means that the table will precompute 2 times
    /// more baby steps than the standard O(sqrt(n)), 1 means symmetrical steps.
    ///
    ///
    /// **NOTE** It is a heavy operation, so pls reuse the same instance for performing
    /// `baby_step_giant_step` function for the same `max_value`.
    pub fn generate_with_balance(
        max_log_value: u64, balance: Option<u64>,
    ) -> Result<Self, BabyStepError> {
        let balance = balance.unwrap_or(DEFAULT_BALANCE);

        if balance == 0 || max_log_value == 0 {
            return Err(BabyStepError::InvalidMaxValueOrBalance(
                max_log_value,
                balance,
            ));
        }

        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let sqrt_step_size = (max_log_value as f64).sqrt().ceil() as u64;
        let baby_step_size = sqrt_step_size * balance;
        let mut table = HashMap::new();

        let mut e = GroupElement::zero();
        for baby_step in 0..=baby_step_size {
            let new_e = &e + &GroupElement::GENERATOR;
            table.insert(e, baby_step);
            e = new_e;
        }

        let giant_step = &GroupElement::GENERATOR * &Scalar::from(baby_step_size).negate();
        Ok(Self {
            table,
            baby_step_size,
            giant_step,
        })
    }

    /// Solve the discrete log using baby step giant step algorithm.
    pub fn discrete_log(&self, mut point: GroupElement) -> Result<u64, BabyStepError> {
        for baby_step in 0..=self.baby_step_size {
            if let Some(x) = self.table.get(&point) {
                let r = baby_step * self.baby_step_size + x;
                return Ok(r);
            }
            point = &point + &self.giant_step;
        }
        // If we get here, the point is not in the table
        // So we exceeded the maximum value of the discrete log
        Err(BabyStepError::MaxLogExceeded)
    }
}
