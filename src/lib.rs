use csv::Writer;
use queue::{Fifo, RRQueue, TaskQueue};
use rand_distr::{Distribution, Exp, ExpError, Poisson, PoissonError};
use std::{collections::BinaryHeap, fmt::Display, path::Path};

mod context;
mod queue;
pub mod task;
use context::Context;
use task::{Task, TaskDefinition};

enum QueueLayer {
    L1,
    L2,
    L3,
}

impl Display for Scheduler<RRQueue, RRQueue, Fifo> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "clock: {},mjr: {}",
            self.clock, self.minimum_job_required
        )?;
        writeln!(
            f,
            "job sync in: {}",
            self.job_sync_period - (self.clock % self.job_sync_period)
        )?;
        writeln!(f, "jobs in pq:")?;
        for task in &self.priority_queue {
            writeln!(f, "{task:?}")?;
        }
        writeln!(f, "jobs in rr1:")?;
        for task in &self.q1.tasks {
            writeln!(f, "{:?}", task)?;
        }
        writeln!(f, "jobs in rr2:")?;
        for task in &self.q2.tasks {
            writeln!(f, "{:?}", task)?;
        }
        writeln!(f, "jobs in fifo:")?;
        for task in &self.q3.tasks {
            writeln!(f, "{:?}", task)?;
        }
        writeln!(f, "finished jobs:")?;
        for task in &self.done {
            writeln!(f, "{:?}", task)?;
        }
        Ok(())
    }
}

pub struct Scheduler<Q1, Q2, Q3> {
    clock: usize,
    job_sync_period: usize,
    minimum_job_required: usize,
    job_creator: JobCreator,
    priority_queue: BinaryHeap<Task>,
    q1: Q1,
    q2: Q2,
    q3: Q3,
    done: Vec<Task>,
    running_task: Option<(Box<dyn Context>, QueueLayer)>,
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
            done: Vec::new(),
            minimum_job_required,
            job_creator: JobCreator::new(arrival_rate, exec_rate)?,
            running_task: None,
        })
    }

    pub fn submit(&mut self, task: TaskDefinition) {
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

        // loading the task
        if self.running_task.is_none() {
            self.running_task = self.dispatch();
        }

        // executing the task
        if let Some((mut job, prev_layer)) = self.running_task.take() {
            self.running_task = match dbg!(job.exec(self.clock)) {
                context::Status::Ready => Some((job, prev_layer)),
                context::Status::TimeOut => {
                    let task = job.take();
                    match prev_layer {
                        QueueLayer::L1 => self.q2.push(task),
                        QueueLayer::L2 => self.q3.push(task),
                        QueueLayer::L3 => unreachable!("since its fifo and has no timeout"),
                    };
                    None
                }
                context::Status::Finished => {
                    self.done.push(job.take());
                    None
                }
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

    fn dispatch(&mut self) -> Option<(Box<dyn Context>, QueueLayer)> {
        if let Some(t) = self.q1.pop() {
            Some((Box::new(t), QueueLayer::L1))
        } else if let Some(t) = self.q2.pop() {
            Some((Box::new(t), QueueLayer::L2))
        } else if let Some(t) = self.q3.pop() {
            Some((Box::new(t), QueueLayer::L3))
        } else {
            None
        }
    }

    pub fn export(&self, path: &Path) {
        let mut wtr = Writer::from_path(path).unwrap();
        for t in &self.done {
            wtr.serialize(t.export());
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
