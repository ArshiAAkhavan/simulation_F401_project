use rand::{thread_rng, Rng};

use crate::{context::Context, queue::TaskQueue, QueueLayer, Scheduler};

pub trait JobDispatcher {
    fn dispatch(&mut self) -> Option<(Box<dyn Context>, QueueLayer)>;
}

pub struct DefaultDispatcher {}
impl JobDispatcher for Scheduler<DefaultDispatcher> {
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
}

pub struct WeightedDispatcher {}
impl JobDispatcher for Scheduler<WeightedDispatcher> {
    fn dispatch(&mut self) -> Option<(Box<dyn Context>, QueueLayer)> {
        let p: f64 = thread_rng().gen();
        // 0 <= p < 1
        if p < 0.8 {
            if let Some(t) = self.q1.pop() {
                return Some((Box::new(t), QueueLayer::L1));
            }
        } else if p < 0.9 {
            if let Some(t) = self.q2.pop() {
                return Some((Box::new(t), QueueLayer::L2));
            }
        } else if let Some(t) = self.q3.pop() {
            return Some((Box::new(t), QueueLayer::L3));
        }
        return None;
    }
}
