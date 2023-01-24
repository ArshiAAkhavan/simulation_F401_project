use std::path::PathBuf;

use simul::{task::TaskDefinition, Scheduler};

const CLEAR: &str = "\x1B[2J\x1B[1;1H";
fn main() {
    let mut sc = match Scheduler::new(20, 5, 2.0, 0.1, 3, 4) {
        Ok(sc) => sc,
        Err(e) => match e {
            simul::SchedulerError::ArrivalRateTooSmall => panic!("arrivalRate should be positive"),
            simul::SchedulerError::ServiceRateTooSmall => {
                panic!("ServiceRate shouldn't be negative")
            }
        },
    };
    sc.submit(TaskDefinition {
        exec_time: 1,
        priority: simul::task::Priority::High,
    });
    for _ in 0..50 {
        // std::io::stdin().read_line(&mut String::new());
        println!("{CLEAR}");
        sc.run();
        println!("{sc}");
    }
    sc.export(&PathBuf::from("res.csv"));
}
