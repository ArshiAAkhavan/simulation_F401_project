#![allow(dead_code)]
use queue::{Fifo, RRQueue, TaskQueue};
use rand_distr::{Distribution, Exp, ExpError, Poisson, PoissonError};
use std::collections::BinaryHeap;

mod context;
mod queue;
mod task;
use context::Context;
use task::{Task, TaskDefinition};

pub struct Scheduler<Q1, Q2, Q3> {
    clock: usize,
    job_sync_period: usize,
    minimum_job_required: usize,
    job_creator: JobCreator,
    priority_queue: BinaryHeap<Task>,
    q1: Q1,
    q2: Q2,
    q3: Q3,
    running_task: Option<Box<dyn Context>>,
}

impl Scheduler<RRQueue, RRQueue, Fifo> {
    pub fn new(
        job_sync_period: usize,
        minimum_job_required: usize,
        arrival_rate: f64,
        exec_rate: f64,
        rr_t1: usize,
        rr_t2: usize,
    ) -> Result<Self, SchedulerError> {
        Ok(Self {
            clock: 0,
            priority_queue: BinaryHeap::new(),
            job_sync_period,
            q1: RRQueue::new(rr_t1),
            q2: RRQueue::new(rr_t2),
            q3: Fifo::new(),
            minimum_job_required,
            job_creator: JobCreator::new(arrival_rate, exec_rate)?,
            running_task: None,
        })
    }

    fn submit(&mut self, task: TaskDefinition) {
        self.priority_queue.push(Task::new_with_priority(
            task.exec_time,
            task.priority,
            self.clock,
        ));
    }

    pub fn run(&mut self) {
        if let Some(task) = self.job_creator.poll() {
            self.submit(task)
        }
        if self.clock % self.job_sync_period == 0 {
            self.sync_jobs()
        }

        if self.running_task.is_none() {
            if let Some(t) = self.q1.pop() {
                self.running_task = Some(Box::new(t))
            } else if let Some(t) = self.q2.pop() {
                self.running_task = Some(Box::new(t))
            } else if let Some(t) = self.q3.pop() {
                self.running_task = Some(Box::new(t))
            }
        }

        if let Some(job) = self.running_task.as_mut() {
            match job.exec(self.clock) {
                context::Status::Ready => (),
                context::Status::TimeOut => todo!(),
                context::Status::Finished => self.running_task = None,
            }
        }

        self.clock += 1;
    }

    fn sync_jobs(&mut self) {
        if self.q1.len() + self.q2.len() + self.q3.len() >= self.minimum_job_required {
            return;
        }
        let mut counter = self.minimum_job_required;
        while let Some(task) = self.priority_queue.pop() {
            self.q1.push(task);
            counter -= 1;
            if counter == 0 {
                break;
            }
        }
    }
}

#[derive(Debug)]
struct JobCreator {
    interval_rnd: Poisson<f64>,
    exectime_rnd: Exp<f64>,
    next_dispatch: usize,
}
impl JobCreator {
    fn new(arrival_rate: f64, exec_rate: f64) -> Result<Self, SchedulerError> {
        Ok(Self {
            interval_rnd: Poisson::new(arrival_rate)?,
            exectime_rnd: Exp::new(exec_rate)?,
            next_dispatch: 0,
        })
    }
    fn poll(&mut self) -> Option<TaskDefinition> {
        if self.next_dispatch == 0 {
            self.next_dispatch = self.interval_rnd.sample(&mut rand::thread_rng()) as usize;
            return Some(TaskDefinition::new(
                self.exectime_rnd.sample(&mut rand::thread_rng()) as usize,
                rand::random(),
            ));
        }
        self.next_dispatch -= 1;
        None
    }
}

pub enum SchedulerError {
    ArrivalRateTooSmall,
    ServiceRateTooSmall,
}

impl From<PoissonError> for SchedulerError {
    fn from(_: PoissonError) -> Self {
        Self::ArrivalRateTooSmall
    }
}

impl From<ExpError> for SchedulerError {
    fn from(_: ExpError) -> Self {
        Self::ServiceRateTooSmall
    }
}
