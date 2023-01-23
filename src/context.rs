use crate::task::{self, Task};

pub enum Status {
    Ready,
    TimeOut,
    Finished,
}

pub trait Context {
    fn exec(&mut self, clock: usize) -> Status;
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
        if self.deadline == 0 {
            return match self.task.status {
                task::Status::Ready => Status::TimeOut,
                task::Status::Finished => Status::Finished,
            };
        }
        self.task.exec(1, clock);
        self.deadline -= 1;
        Status::Ready
    }
}
impl Context for DefaultContext {
    fn exec(&mut self, clock: usize) -> Status {
        match self.task.status {
            task::Status::Ready => {
                self.task.exec(1, clock);
                Status::Ready
            }
            task::Status::Finished => Status::Finished,
        }
    }
}

impl From<DeadlineContext> for Task {
    fn from(value: DeadlineContext) -> Self {
        value.task
    }
}
impl From<DefaultContext> for Task {
    fn from(value: DefaultContext) -> Self {
        value.task
    }
}
