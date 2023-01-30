use csv::Writer;
use job_creator::JobCreator;
use job_dispatcher::{DefaultDispatcher, WeightedDispatcher};
use queue::{Fifo, RRQueue, TaskQueue};
use std::{collections::BinaryHeap, fmt::Display, marker::PhantomData, path::Path};

mod context;
mod error;
mod job_creator;
mod job_dispatcher;
mod queue;
use context::Context;
pub mod task;
pub use error::SchedulerError;
pub use job_dispatcher::JobDispatcher;
use task::{Task, TaskDefinition};

pub enum QueueLayer {
    L1,
    L2,
    L3,
}

impl<D> Display for Scheduler<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "clock: {},mjr: {}", self.clock, self.job_threshold)?;
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

pub struct Scheduler<Dispatcher> {
    clock: usize,
    job_sync_period: usize,
    job_threshold: usize,
    job_creator: JobCreator,
    priority_queue: BinaryHeap<Task>,
    q1: RRQueue,
    q2: RRQueue,
    q3: Fifo,
    done: Vec<Task>,
    running_task: Option<(Box<dyn Context>, QueueLayer)>,
    phantom: PhantomData<Dispatcher>,
}
impl Scheduler<DefaultDispatcher> {
    pub fn new(
        job_sync_period: usize,
        job_threshold: usize,
        arrival_rate: f64,
        exec_rate: f64,
        rr_t1: usize,
        rr_t2: usize,
        timeout_rate: Option<f64>,
    ) -> Result<Self, SchedulerError> {
        Ok(Self {
            clock: 0,
            priority_queue: BinaryHeap::new(),
            job_sync_period,
            q1: RRQueue::new(rr_t1),
            q2: RRQueue::new(rr_t2),
            q3: Fifo::new(),
            done: Vec::new(),
            phantom: PhantomData,
            job_threshold,
            job_creator: JobCreator::new(arrival_rate, exec_rate, timeout_rate)?,
            running_task: None,
        })
    }
    pub fn with_weighted_dispatcher(self) -> Scheduler<WeightedDispatcher> {
        Scheduler {
            clock: self.clock,
            priority_queue: self.priority_queue,
            job_sync_period: self.job_sync_period,
            q1: self.q1,
            q2: self.q2,
            q3: self.q3,
            done: self.done,
            phantom: PhantomData,
            job_threshold: self.job_threshold,
            job_creator: self.job_creator,
            running_task: self.running_task,
        }
    }
}

impl<Dispatcher> Scheduler<Dispatcher>
where
    Scheduler<Dispatcher>: JobDispatcher,
{
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
            self.running_task = match job.exec(self.clock) {
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
}

impl<Dispatcher> Scheduler<Dispatcher> {
    fn submit(&mut self, task: TaskDefinition) {
        self.priority_queue.push(Task::new(
            task.exec_time,
            task.priority,
            self.clock,
            task.timeout,
        ));
    }

    fn sync_jobs(&mut self) {
        if self.q1.len() + self.q2.len() + self.q3.len() >= self.job_threshold {
            return;
        }
        let mut counter = self.job_threshold;
        while let Some(mut task) = self.priority_queue.pop() {
            task.set_schedule_time(self.clock);
            self.q1.push(task);
            counter -= 1;
            if counter == 0 {
                break;
            }
        }
    }

    pub fn export(&self, path: &Path) {
        let mut wtr = Writer::from_path(path).unwrap();
        for t in self
            .done
            .iter()
            .chain(self.q1.tasks.iter())
            .chain(self.q2.tasks.iter())
            .chain(self.q3.tasks.iter())
            .chain(self.priority_queue.iter())
        {
            let _ = wtr.serialize(t.export());
        }
    }
}
