use std::collections::VecDeque;

use crate::{
    context::{Context, DeadlineContext, DefaultContext},
    task::Task,
};

pub trait TaskQueue<Ctx>
where
    Ctx: Context,
{
    fn push(&mut self, task: Task);
    fn pop(&mut self) -> Option<Ctx>;
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
impl TaskQueue<DeadlineContext> for RRQueue {
    fn push(&mut self, task: Task) {
        self.tasks.push_back(task)
    }

    fn pop(&mut self) -> Option<DeadlineContext> {
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

impl TaskQueue<DefaultContext> for Fifo {
    fn push(&mut self, task: Task) {
        self.tasks.push(task)
    }

    fn pop(&mut self) -> Option<DefaultContext> {
        self.tasks.pop().map(|t| t.with_context())
    }

    fn len(&self) -> usize {
        self.tasks.len()
    }
}
