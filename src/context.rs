use crate::task::{self, Task};

#[derive(Debug)]
pub enum Status {
    Ready,
    TimeOut,
    Finished,
}

pub trait Context {
    fn exec(&mut self, clock: usize) -> Status;
    fn take(self: Box<Self>) -> Task;
}

pub struct DeadlineContext {
    deadline: usize,
    task: Task,
}

impl DeadlineContext {
    pub(super) fn new(task: Task, deadline: usize) -> Self {
        Self { deadline, task }
    }
}

pub struct DefaultContext {
    task: Task,
}

impl DefaultContext {
    pub(super) fn new(task: Task) -> Self {
        Self { task }
    }
}

impl Context for DeadlineContext {
    fn exec(&mut self, clock: usize) -> Status {
        println!("deadline: {}", self.deadline);
        println!("{:?}", self.task);
        println!("{:?}", self.task.progress);
        if self.deadline == 0 {
            return match self.task.status {
                task::Status::Ready => Status::TimeOut,
                task::Status::Finished => Status::Finished,
            };
        }
        self.task.exec(clock);
        self.deadline -= 1;
        if self.task.status == task::Status::Finished {
            return Status::Finished;
        }
        Status::Ready
    }

    fn take(self: Box<Self>) -> Task {
        self.task
    }
}
impl Context for DefaultContext {
    fn exec(&mut self, clock: usize) -> Status {
        println!("{:?}", self.task);
        println!("{:?}", self.task.progress);
        match self.task.status {
            task::Status::Ready => {
                self.task.exec(clock);
                Status::Ready
            }
            task::Status::Finished => Status::Finished,
        }
    }

    fn take(self: Box<Self>) -> Task {
        self.task
    }
}
