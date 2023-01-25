use rand_distr::{ExpError, PoissonError};

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
