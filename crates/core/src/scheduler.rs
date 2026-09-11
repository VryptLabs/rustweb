use std::collections::VecDeque;
use std::fmt::Debug;

pub const MAX_MSGS_PER_TICK: usize = 1024;

#[derive(Debug, Default)]
pub struct Scheduler<M: Debug> {
    queue: VecDeque<M>,

    pub enqueued: u64,

    pub dropped: u64,
}

impl<M: Debug> Scheduler<M> {

    pub fn new() -> Self {
        Self { queue: VecDeque::new(), enqueued: 0, dropped: 0 }
    }

    pub fn push(&mut self, msg: M) {
        self.queue.push_back(msg);
        self.enqueued += 1;
    }

    pub fn pop(&mut self) -> Option<M> {
        self.queue.pop_front()
    }

    pub fn pending(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn drain<E>(&mut self, mut f: impl FnMut(M) -> Result<(), E>) -> Result<usize, E> {
        let mut n = 0;
        while let Some(msg) = self.queue.pop_front() {
            if n >= MAX_MSGS_PER_TICK {
                self.dropped += 1 + self.queue.len() as u64;
                self.queue.clear();
                break;
            }
            f(msg)?;
            n += 1;
        }
        Ok(n)
    }

    pub fn stats(&self) -> SchedulerStats {
        SchedulerStats { pending: self.pending(), enqueued: self.enqueued, dropped: self.dropped }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchedulerStats {

    pub pending: usize,

    pub enqueued: u64,

    pub dropped: u64,
}
