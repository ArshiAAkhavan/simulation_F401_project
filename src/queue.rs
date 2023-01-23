use std::collections::VecDeque;

use crate::{
    context::{Context, DeadlineContext, DefaultContext},
    task::Task,
};

pub trait TaskQueue {
    type Ctx: Context;

    fn push(&mut self, task: Task);
    fn pop(&mut self) -> Option<Self::Ctx>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub struct RRQueue {
    timeslice: usize,
    tasks: VecDeque<Task>,
}
impl RRQueue {
    pub fn new(timeslice: usize) -> Self {
        Self {
            timeslice,
            tasks: Default::default(),
        }
    }
}
impl TaskQueue for RRQueue {
    type Ctx = DeadlineContext;
    fn push(&mut self, task: Task) {
        self.tasks.push_back(task)
    }

    fn pop(&mut self) -> Option<Self::Ctx> {
        self.tasks
            .pop_front()
            .map(|t| t.with_deadline(self.timeslice))
    }

    fn len(&self) -> usize {
        self.tasks.len()
    }
}

#[derive(Default)]
pub struct Fifo {
    tasks: Vec<Task>,
}

impl Fifo {
    pub fn new() -> Self {
        Self::default()
    }
}

impl TaskQueue for Fifo {
    type Ctx = DefaultContext;
    fn push(&mut self, task: Task) {
        self.tasks.push(task)
    }

    fn pop(&mut self) -> Option<Self::Ctx> {
        self.tasks.pop().map(|t| t.with_context())
    }

    fn len(&self) -> usize {
        self.tasks.len()
    }
}
