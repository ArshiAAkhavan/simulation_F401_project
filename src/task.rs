use std::cmp::{min, Ordering};

use rand_distr::{Distribution, Standard};

use crate::context::{DefaultContext, DeadlineContext};

#[derive(Debug, PartialEq, Eq)]
pub struct Task {
    exec_time: usize,
    arival_time: usize,
    remaining: usize,
    pub priority: Priority,
    pub status: Status,
    progress: Vec<(usize, usize)>,
}
impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(match self.priority.cmp(&other.priority) {
            Ordering::Less => Ordering::Less,
            Ordering::Equal => self.arival_time.cmp(&other.arival_time),
            Ordering::Greater => Ordering::Greater,
        })
    }
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.priority.cmp(&other.priority) {
            Ordering::Less => Ordering::Less,
            Ordering::Equal => self.arival_time.cmp(&other.arival_time),
            Ordering::Greater => Ordering::Greater,
        }
    }
}

impl Task {
    pub fn new_with_priority(exec_time: usize, priority: Priority, arival_time: usize) -> Self {
        Self {
            exec_time,
            arival_time,
            remaining: exec_time,
            priority,
            status: Status::Ready,
            progress: Vec::new(),
        }
    }

    pub fn exec(&mut self, amount: usize, clock: usize) {
        self.progress
            .push((clock, clock + min(amount, self.remaining)));
        self.remaining = self.remaining.checked_sub(amount).unwrap_or(0);
        if self.remaining == 0 {
            self.status = Status::Finished
        } else {
            self.status = Status::Ready
        }
    }
    pub fn with_deadline(self, deadline: usize) -> DeadlineContext {
        DeadlineContext::new(self, deadline)
    }

    pub fn with_context(self) -> DefaultContext {
        DefaultContext::new(self)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Status {
    Ready,
    Finished,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
}
impl Distribution<Priority> for Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Priority {
        match rng.gen::<f64>() {
            x if x < 0.7 => Priority::Low,
            x if 0.7 <= x && x < 0.9 => Priority::Normal,
            x if 0.9 <= x && x < 1.0 => Priority::High,
            _ => unreachable!(),
        }
    }
}

pub struct TaskDefinition {
    pub exec_time: usize,
    pub priority: Priority,
}

impl TaskDefinition {
    pub fn new(exec_time: usize, priority: Priority) -> Self {
        Self {
            exec_time,
            priority,
        }
    }
}

