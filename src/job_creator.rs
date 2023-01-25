use rand_distr::{Distribution, Exp, Poisson};

use crate::error::SchedulerError;
use crate::task::TaskDefinition;

#[derive(Debug)]
pub(crate) struct JobCreator {
    interval_rnd: Poisson<f64>,
    exectime_rnd: Exp<f64>,
    next_dispatch: usize,
}
impl JobCreator {
    pub(crate) fn new(arrival_rate: f64, exec_rate: f64) -> Result<Self, SchedulerError> {
        Ok(Self {
            interval_rnd: Poisson::new(arrival_rate)?,
            exectime_rnd: Exp::new(exec_rate)?,
            next_dispatch: 0,
        })
    }
    pub(crate) fn poll(&mut self) -> Option<TaskDefinition> {
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
