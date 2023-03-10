use std::cmp::Ordering;

use rand_distr::{Distribution, Standard};
use serde::Serialize;

use crate::context::{DeadlineContext, DefaultContext};

#[derive(Debug, PartialEq, Eq)]
pub struct Task {
    exec_time: usize,
    arrival_time: usize,
    schedule_time: Option<usize>,
    remaining: usize,
    priority: Priority,
    pub status: Status,
    progress: Vec<usize>,
    timeout: Option<usize>,
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
    pub fn new(
        exec_time: usize,
        priority: Priority,
        arival_time: usize,
        timeout: Option<usize>,
    ) -> Self {
        Self {
            exec_time,
            arrival_time: arival_time,
            schedule_time: None,
            remaining: exec_time,
            priority,
            status: Status::Ready,
            progress: Vec::new(),
            timeout,
        }
    }
    pub fn set_schedule_time(&mut self, time: usize) {
        self.schedule_time = Some(time);
    }

    pub fn exec(&mut self, clock: usize) {
        if let Some(timeout) = self.timeout {
            if self.arrival_time + timeout < clock {
                self.status = Status::TimeOut;
                return;
            }
        }

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
        let service_start = self.progress.first().copied().unwrap_or_default();
        let service_end = match self.status {
            Status::Ready | Status::Finished => self.progress.last().copied().unwrap_or_default(),
            Status::TimeOut => self.arrival_time + self.timeout.unwrap(),
        };
        TaskRecord {
            service_start,
            service_end,
            arrival_time: self.arrival_time,
            schedule_time: self.schedule_time.unwrap_or_default(),
            service_time: self.progress.len(),
            exec_time: self.exec_time,
            priority: format!("{:?}", self.priority),
            status: format!("{:?}", self.status),
        }
    }
}

#[derive(Serialize)]
pub struct TaskRecord {
    service_start: usize,
    service_end: usize,
    arrival_time: usize,
    schedule_time: usize,
    service_time: usize,
    exec_time: usize,
    priority: String,
    status: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Status {
    Ready,
    Finished,
    TimeOut,
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
            x if (0.0..0.7).contains(&x) => Priority::Low,
            x if (0.7..0.9).contains(&x) => Priority::Normal,
            x if (0.9..1.0).contains(&x) => Priority::High,
            _ => unreachable!(),
        }
    }
}

pub struct TaskDefinition {
    pub exec_time: usize,
    pub priority: Priority,
    pub timeout: Option<usize>,
}

impl TaskDefinition {
    pub fn new(exec_time: usize, priority: Priority, timeout: Option<usize>) -> Self {
        Self {
            exec_time,
            priority,
            timeout,
        }
    }
}
