use std::collections::LinkedList;

use crate::utils::types::Milliseconds;

/// Keeps track of the frame rate across some period as an average.
#[derive(Debug)]
pub struct FpsTracker {
    times: LinkedList<u32>,
    period: Milliseconds,
}

impl Default for FpsTracker {
    fn default() -> Self {
        Self {
            times: LinkedList::new(),
            period: 3,
        }
    }
}

impl FpsTracker {
    pub fn with_tracking_period(period: Milliseconds) -> Self {
        Self {
            times: LinkedList::new(),
            period,
        }
    }

    /// Assumes values passed in across multiple calls are monotonically increasing. Returns the
    /// delta between now and the last tick call
    pub fn tick(&mut self, now: Milliseconds) -> Milliseconds {
        let delta = now - self.times.back().unwrap_or(&now);
        self.times.push_back(now);
        // assumes list is sorted in ascending order
        while let Some(time) = self.times.front()
            && *time + self.period < now
        {
            self.times.pop_front();
        }

        delta
    }

    pub fn fps(&self) -> u32 {
        let mut iter = self.times.iter().peekable();
        let mut sum = 0;
        while let Some(curr) = iter.next()
            && let Some(next) = iter.peek()
        {
            sum += *next - curr;
        }

        (sum as f32 / self.times.len() as f32).round() as u32
    }
}
