//! SP3 file merging operations.

use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum MergeError {}

pub trait Merge {
    fn merge(&self, rhs: &Self) -> Result<Self, MergeError>
    where
        Self: Sized;
    fn merge_mut(&mut self, rhs: &Self) -> Result<(), MergeError>
    where
        Self: Sized;
}
