use std::rc::Rc;

use chrono::{DateTime, Utc};

use super::error::AppError;

/// Data from a successful fetch and when it arrived.
#[derive(Debug, PartialEq)]
pub struct Snapshot<T> {
    pub data: Rc<T>,
    pub fetched_at: DateTime<Utc>,
}

// Manual impl: cloning only bumps the `Rc`, so `T` need not be `Clone`.
impl<T> Clone for Snapshot<T> {
    fn clone(&self) -> Self {
        Self {
            data: Rc::clone(&self.data),
            fetched_at: self.fetched_at,
        }
    }
}

/// Loading state for remotely fetched data.
#[derive(Debug, PartialEq)]
pub enum LoadState<T> {
    /// Nothing to show yet; the first fetch is in flight.
    Loading,
    /// The latest fetch succeeded.
    Loaded(Snapshot<T>),
    /// The latest fetch failed, but earlier data is still available.
    Stale {
        snapshot: Snapshot<T>,
        error: AppError,
    },
    /// The first fetch failed and there is nothing to show.
    Failed(AppError),
}

impl<T> Clone for LoadState<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Loading => Self::Loading,
            Self::Loaded(snapshot) => Self::Loaded(snapshot.clone()),
            Self::Stale { snapshot, error } => Self::Stale {
                snapshot: snapshot.clone(),
                error: error.clone(),
            },
            Self::Failed(error) => Self::Failed(error.clone()),
        }
    }
}

impl<T> LoadState<T> {
    /// The most recent successful fetch, if any.
    pub const fn snapshot(&self) -> Option<&Snapshot<T>> {
        match self {
            Self::Loaded(snapshot) | Self::Stale { snapshot, .. } => Some(snapshot),
            Self::Loading | Self::Failed(_) => None,
        }
    }

    /// The most recently fetched data, if any.
    pub fn data(&self) -> Option<&Rc<T>> {
        self.snapshot().map(|snapshot| &snapshot.data)
    }

    /// The error from the latest fetch, if it failed.
    pub const fn error(&self) -> Option<&AppError> {
        match self {
            Self::Stale { error, .. } | Self::Failed(error) => Some(error),
            Self::Loading | Self::Loaded(_) => None,
        }
    }

    /// Folds a fetch result into this state, keeping the last good data when a refresh fails.
    pub fn apply(&self, result: Result<T, AppError>, fetched_at: DateTime<Utc>) -> Self {
        match (result, self.snapshot()) {
            (Ok(data), _) => Self::Loaded(Snapshot {
                data: Rc::new(data),
                fetched_at,
            }),
            (Err(error), Some(snapshot)) => Self::Stale {
                snapshot: snapshot.clone(),
                error,
            },
            (Err(error), None) => Self::Failed(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(hour: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, 1, hour, 0, 0).unwrap()
    }

    #[test]
    fn success_replaces_any_previous_state() {
        let state = LoadState::<u32>::Failed(AppError::Timeout).apply(Ok(7), at(1));

        assert_eq!(state.data().map(|data| **data), Some(7));
        assert_eq!(state.snapshot().unwrap().fetched_at, at(1));
        assert_eq!(state.error(), None);
    }

    #[test]
    fn failed_refresh_keeps_last_good_data() {
        let loaded = LoadState::Loading.apply(Ok(7), at(1));
        let stale = loaded.apply(Err(AppError::Timeout), at(2));

        assert_eq!(stale.data().map(|data| **data), Some(7));
        assert_eq!(stale.snapshot().unwrap().fetched_at, at(1));
        assert_eq!(stale.error(), Some(&AppError::Timeout));
    }

    #[test]
    fn failure_without_data_is_failed() {
        let state = LoadState::<u32>::Loading.apply(Err(AppError::Http(503)), at(1));

        assert_eq!(state, LoadState::Failed(AppError::Http(503)));
        assert!(state.data().is_none());
    }
}
