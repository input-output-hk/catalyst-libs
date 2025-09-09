//! Implementation of baby-step giant-step algorithm to solve the discrete logarithm over
//! for the Ristretto255 group.

use std::collections::HashMap;

use anyhow::{bail, ensure};

use crate::crypto::group::{GroupElement, Scalar};

/// Default balance value.
/// Make steps asymmetric, in order to better use caching of baby steps.
/// Balance of 2 means that baby steps are 2 time more than `sqrt(max_votes)`
const DEFAULT_BALANCE: u64 = 2;

/// Holds precomputed baby steps `table` for the baby-step giant-step algorithm
/// for solving discrete log.
#[derive(Debug, Clone)]
pub struct BabyStepGiantStep {
    /// Table of baby step precomputed values
    table: HashMap<GroupElement, u64>,
    /// baby step size value
    baby_step_size: u64,
    /// giant step value
    giant_step: GroupElement,
}

impl BabyStepGiantStep {
    /// Creates a new setup for the baby-step giant-step algorithm.
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
    ///
    /// # Errors
    ///   - Maximum value and balance must be greater than zero.
    pub fn new(
        max_log_value: u64,
        balance: Option<u64>,
    ) -> anyhow::Result<Self> {
        let balance = balance.unwrap_or(DEFAULT_BALANCE);

        ensure!(
            balance != 0 && max_log_value != 0,
            "Maximum value and balance must be greater than zero,
            provided max value: {max_log_value} and balance: {balance}."
        );

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
    ///
    /// # Errors
    ///   - Max log value exceeded.
    pub fn discrete_log(
        &self,
        mut point: GroupElement,
    ) -> anyhow::Result<u64> {
        for baby_step in 0..=self.baby_step_size {
            if let Some(x) = self.table.get(&point) {
                let r = baby_step * self.baby_step_size + x;
                return Ok(r);
            }
            point = &point + &self.giant_step;
        }

        // If we get here, the point is not in the table
        // So we exceeded the maximum value of the discrete log
        bail!("Max log value exceeded.
                Means that the actual discrete log for the provided group element is higher than the provided `max_log_value`."
        )
    }
}

#[cfg(test)]
#[allow(clippy::explicit_deref_methods)]
mod tests {
    use std::ops::Mul;

    use test_strategy::proptest;

    use super::*;

    // Starting `max_log_value` from 2 allows to eliminate possible `Invalid use of empty
    // range 1..1` for `log` strategy
    #[proptest]
    fn baby_step_giant_step_test(
        #[strategy(2..10000u64)] max_log_value: u64,
        #[strategy(1..#max_log_value)] log: u64,
    ) {
        let ge = GroupElement::GENERATOR.mul(&Scalar::from(log));

        let baby_step_giant_step = BabyStepGiantStep::new(max_log_value, None).unwrap();
        let result = baby_step_giant_step.discrete_log(ge).unwrap();
        assert_eq!(result, log);
    }
}
