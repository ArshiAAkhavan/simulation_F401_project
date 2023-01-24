use std::cmp::Ordering;

use rand_distr::{Distribution, Standard};
use serde::Serialize;

use crate::context::{DeadlineContext, DefaultContext};

#[derive(Debug, PartialEq, Eq)]
pub struct Task {
    exec_time: usize,
    arrival_time: usize,
    remaining: usize,
    pub priority: Priority,
    pub status: Status,
    pub progress: Vec<usize>,
}
impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(match self.priority.cmp(&other.priority) {
            Ordering::Less => Ordering::Less,
            Ordering::Equal => self.arrival_time.cmp(&other.arrival_time),
            Ordering::Greater => Ordering::Greater,
        })
    }
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.priority.cmp(&other.priority) {
            Ordering::Less => Ordering::Less,
            Ordering::Equal => self.arrival_time.cmp(&other.arrival_time),
            Ordering::Greater => Ordering::Greater,
        }
    }
}

impl Task {
    pub fn new_with_priority(exec_time: usize, priority: Priority, arival_time: usize) -> Self {
        Self {
            exec_time,
            arrival_time: arival_time,
            remaining: exec_time,
            priority,
            status: Status::Ready,
            progress: Vec::new(),
        }
    }

    pub fn exec(&mut self, clock: usize) {
        self.progress.push(clock);
        self.remaining = self.remaining.saturating_sub(1);
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

    pub fn export(&self) -> TaskRecord {
        TaskRecord {
            service_start: self.progress.first().copied().unwrap_or_default(),
            service_end: self.progress.last().copied().unwrap_or_default(),
            arrival_time: self.arrival_time,
            service_time: self.progress.len(),
            exec_time: self.exec_time,
            priority: format!("{:?}", self.priority),
        }
    }
}

#[derive(Serialize)]
pub struct TaskRecord {
    service_start: usize,
    service_end: usize,
    arrival_time: usize,
    service_time: usize,
    exec_time: usize,
    priority: String,
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
            0.0..=0.7 => Priority::Low,
            0.7..=0.9 => Priority::Normal,
            0.9..=1.0 => Priority::High,
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
