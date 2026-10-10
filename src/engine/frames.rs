use std::collections::LinkedList;

use crate::utils::types::Milliseconds;

/// Keeps track of the frame rate across some period as an average.
#[derive(Debug)]
pub struct FpsTracker {
    times: LinkedList<Milliseconds>,
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

    /// Returns the delta between now and the last tick call
    pub fn tick(&mut self, now: Milliseconds) -> Milliseconds {
        let last = self.times.back().copied().unwrap_or_default();
        assert!(now >= last, "failed: now: {now} > last: {last}");
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
        let times = self.frame_times();
        let avg = times.iter().map(|t| *t as f32).sum::<f32>() / times.len() as f32;
        (1000.0 / avg) as u32
    }

    fn frame_times(&self) -> Vec<Milliseconds> {
        let mut iter = self.times.iter().peekable();
        let mut times = Vec::with_capacity(self.times.len() - 1);
        while let Some(curr) = iter.next()
            && let Some(next) = iter.peek()
        {
            times.push(*next - curr);
        }

        times
    }
}
