//! Spreads work over the cores when the `parallel` feature is on, and runs it in turn otherwise.

/// Calls `work` on every item and returns the results in the order of the items, whatever order the calls ran in.
#[cfg(feature = "parallel")]
pub(super) fn map<T: Sync, R: Send>(items: &[T], work: impl Fn(&T) -> R + Sync + Send) -> Vec<R> {
    use rayon::prelude::*;
    items.par_iter().map(work).collect()
}

/// Calls `work` on every item, in turn, and returns the results in the order of the items.
#[cfg(not(feature = "parallel"))]
pub(super) fn map<T: Sync, R: Send>(items: &[T], work: impl Fn(&T) -> R + Sync + Send) -> Vec<R> {
    items.iter().map(work).collect()
}
