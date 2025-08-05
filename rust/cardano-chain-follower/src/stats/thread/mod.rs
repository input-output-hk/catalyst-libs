//! Thread statistics.

pub(crate) mod name;

use std::{
    fmt,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use cpu_time::ThreadTime;
use serde::{
    ser::{SerializeStruct, Serializer},
    Serialize,
};
use tracing::error;

/// Thread statistics.
#[derive(Debug, Default, Clone, Serialize)]
#[allow(clippy::module_name_repetitions)]
pub struct ThreadStat(Arc<InnerThreadStat>);

/// Inner thread statistics.
struct InnerThreadStat {
    /// A counter for the number of times the thread has been resumed.
    counter: AtomicU64,
    /// A boolean value indicating whether the thread is running.
    is_running: AtomicBool,
    /// A boolean value indicating whether the thread is a service.
    is_service: AtomicBool,
    /// The latest CPU time.
    latest_cpu_time: Mutex<Duration>,
    /// The total CPU time.
    total_cpu_time: Mutex<Duration>,
}

impl Serialize for InnerThreadStat {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ThreadStat", 6)?;

        state.serialize_field("counter", &self.counter.load(Ordering::SeqCst))?;
        state.serialize_field("is_running", &self.is_running.load(Ordering::SeqCst))?;
        state.serialize_field("is_service", &self.is_service.load(Ordering::SeqCst))?;
        if let Ok(total_cpu_time) = self.total_cpu_time.lock() {
            state.serialize_field("total_cpu_time", &*total_cpu_time)?;
        }
        if let Ok(latest_cpu_time) = self.latest_cpu_time.lock() {
            state.serialize_field("latest_cpu_time", &*latest_cpu_time)?;
        }
        state.end()
    }
}

impl fmt::Debug for InnerThreadStat {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let latest_cpu_time = self.latest_cpu_time.lock().map_err(|_| fmt::Error)?;
        let total_cpu_time = self.total_cpu_time.lock().map_err(|_| fmt::Error)?;

        f.debug_struct("InnerThreadStat")
            .field("counter", &self.counter.load(Ordering::SeqCst))
            .field("is_running", &self.is_running.load(Ordering::SeqCst))
            .field("is_service", &self.is_service.load(Ordering::SeqCst))
            .field("latest_cpu_time", &*latest_cpu_time)
            .field("total_cpu_time", &*total_cpu_time)
            .finish()
    }
}

impl Default for InnerThreadStat {
    fn default() -> Self {
        InnerThreadStat {
            counter: AtomicU64::new(0),
            is_running: AtomicBool::new(false),
            is_service: AtomicBool::new(false),
            latest_cpu_time: Mutex::new(Duration::ZERO),
            total_cpu_time: Mutex::new(Duration::ZERO),
        }
    }
}

impl InnerThreadStat {
    /// Update the total time of the CPU used.
    fn update_total_time(&self) {
        // Get the current CPU time as a Duration
        let current_time = ThreadTime::now().as_duration();

        if let Ok(latest_cpu_time) = self.latest_cpu_time.lock() {
            // Calculate elapsed time (current - previous)
            if let Some(elapsed) = current_time.checked_sub(*latest_cpu_time) {
                // If the elapsed time is non-negative, update total_cpu_time
                if let Ok(mut total_cpu_time) = self.total_cpu_time.lock() {
                    if let Some(sum) = total_cpu_time.checked_add(elapsed) {
                        *total_cpu_time = sum;
                    } else {
                        error!("Total CPU time overflow");
                    }
                }
            }
        }
    }

    /// Update the latest time of the CPU used.
    fn update_latest_time(&self) {
        if let Ok(mut latest_cpu_time) = self.latest_cpu_time.lock() {
            *latest_cpu_time = ThreadTime::now().as_duration();
        }
    }

    /// Increase the counter by 1.
    fn increment_counter(&self) {
        self.counter.fetch_add(1, Ordering::SeqCst);
    }
}

impl ThreadStat {
    /// Initialize a thread statistic.
    pub(crate) fn start_thread(is_service: bool) -> Self {
        Self(Arc::new(InnerThreadStat {
            counter: 0.into(),
            is_running: true.into(),
            is_service: is_service.into(),
            total_cpu_time: Duration::ZERO.into(),
            latest_cpu_time: ThreadTime::now().as_duration().into(),
        }))
    }

    /// Stop the thread.
    pub(crate) fn stop_thread(&self) {
        self.0.is_running.store(false, Ordering::SeqCst);
        self.0.update_latest_time();
        self.0.update_total_time();
    }

    /// Resume the thread.
    pub(crate) fn resume_thread(&self) {
        self.0.increment_counter();
        self.0.update_latest_time();
    }

    /// Pause the thread.
    pub(crate) fn pause_thread(&self) {
        self.0.update_latest_time();
        self.0.update_total_time();
    }

    /// Is the thread running?
    pub fn is_running(&self) -> bool {
        self.0.is_running.load(Ordering::SeqCst)
    }

    /// The number of times the thread has been resumed.
    pub fn counter(&self) -> u64 {
        self.0.counter.load(Ordering::SeqCst)
    }

    /// Get the total CPU time for a thread.
    pub fn total_cpu_time(&self) -> Option<Duration> {
        self.0
            .total_cpu_time
            .lock()
            .ok()
            .map(|total_cpu_time| *total_cpu_time)
    }

    /// Get the latest CPU time for a thread.
    pub fn latest_cpu_time(&self) -> Option<Duration> {
        self.0
            .latest_cpu_time
            .lock()
            .ok()
            .map(|latest_cpu_time| *latest_cpu_time)
    }
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use super::*;

    #[test]
    fn test_thread_stat_initialization() {
        let stat = ThreadStat::start_thread(true);
        assert!(stat.is_running());
        assert_eq!(stat.counter(), 0);
        assert!(stat.total_cpu_time().is_some());
        assert!(stat.latest_cpu_time().is_some());
    }

    #[test]
    fn test_thread_stat_stop() {
        let stat = ThreadStat::start_thread(false);
        stat.stop_thread();
        assert!(!stat.is_running());
    }

    #[test]
    fn test_thread_stat_resume() {
        let stat = ThreadStat::start_thread(false);
        stat.resume_thread();
        assert!(stat.is_running());
        assert_eq!(stat.counter(), 1);
    }

    #[test]
    fn test_thread_stat_pause() {
        let stat = ThreadStat::start_thread(false);
        stat.pause_thread();
        assert!(stat.is_running());
    }

    #[test]
    fn test_thread_stat_update_cpu_time() {
        let stat = ThreadStat::start_thread(false);
        stat.pause_thread();
        thread::sleep(Duration::from_millis(10));
        stat.resume_thread();
        let total_cpu_time = stat.total_cpu_time().unwrap();
        assert!(total_cpu_time > Duration::ZERO);
    }

    #[test]
    fn test_thread_stat_multiple_resumes() {
        let stat = ThreadStat::start_thread(false);
        stat.resume_thread();
        stat.resume_thread();
        stat.resume_thread();
        assert_eq!(stat.counter(), 3);
    }
}
