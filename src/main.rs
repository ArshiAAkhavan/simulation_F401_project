use simul::Scheduler;
fn main() {
    let mut sc = match Scheduler::new(20, 5, 2.0, 10.0, 3, 4) {
        Ok(sc) => sc,
        Err(e) => match e {
            simul::SchedulerError::ArrivalRateTooSmall => panic!("arrivalRate should be positive"),
            simul::SchedulerError::ServiceRateTooSmall => {
                panic!("ServiceRate shouldn't be negative")
            }
        },
    };
    for _ in 0..50{
        sc.run();
    }
}
