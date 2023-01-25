use rand_distr::{Distribution, Exp, Poisson};

use crate::error::SchedulerError;
use crate::task::TaskDefinition;

#[derive(Debug)]
pub(crate) struct JobCreator {
    interval_rnd: Poisson<f64>,
    exectime_rnd: Exp<f64>,
    timeout_rnd: Option<Exp<f64>>,
    next_dispatch: usize,
}
impl JobCreator {
    pub(crate) fn new(
        arrival_rate: f64,
        exec_rate: f64,
        timeout_rate: Option<f64>,
    ) -> Result<Self, SchedulerError> {
        let timeout_rnd = if let Some(timeout_rate) = timeout_rate {
            Some(Exp::new(timeout_rate)?)
        } else {
            None
        };
        Ok(Self {
            interval_rnd: Poisson::new(arrival_rate)?,
            exectime_rnd: Exp::new(exec_rate)?,
            timeout_rnd,
            next_dispatch: 0,
        })
    }
    pub(crate) fn poll(&mut self) -> Option<TaskDefinition> {
        if self.next_dispatch == 0 {
            self.next_dispatch = self.interval_rnd.sample(&mut rand::thread_rng()) as usize;
            let timeout = match self.timeout_rnd {
                Some(exp) => Some(exp.sample(&mut rand::thread_rng()) as usize),
                None => None,
            };
            return Some(TaskDefinition::new(
                self.exectime_rnd.sample(&mut rand::thread_rng()) as usize,
                rand::random(),
                timeout,
            ));
        }
        self.next_dispatch -= 1;
        None
    }
}
