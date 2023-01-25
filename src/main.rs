use std::path::PathBuf;

use clap::Parser;
use simul::{task::TaskDefinition, JobDispatcher, Scheduler};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// cycle of which we transfer jobs from first stage to the second stage
    #[arg(id = "sync_period", short, long)]
    job_sync_period: usize,

    /// threshold of the jobs in the second stage
    #[arg(short, long)]
    job_threshold: usize,

    /// arrival rate of the jobs
    #[arg(short, long)]
    arrival_rate: f64,

    /// execution rate of the jobs
    #[arg(short, long)]
    exec_rate: f64,

    /// quantom time for the first Round Robin queue
    #[arg(id = "t2", long)]
    rrt1: usize,

    /// quantom time for the second Round Robin queue
    #[arg(id = "t1", long)]
    rrt2: usize,

    /// use the weighted dispatcher instead of priority dispatcher
    #[arg(short, long)]
    weighted_dispatcher: bool,

    /// set timeout rate of each task, regardless of its queue.
    /// if not set, task would run without any timeout
    #[arg(short, long)]
    timeout_rate: Option<f64>,

    /// test duration
    #[arg(short, long)]
    duration: usize,
}

const CLEAR: &str = "\x1B[2J\x1B[1;1H";
fn main() {
    let opt = Cli::parse();

    let mut sc = match Scheduler::new(
        opt.job_sync_period,
        opt.job_threshold,
        opt.arrival_rate,
        opt.exec_rate,
        opt.rrt1,
        opt.rrt2,
        opt.timeout_rate,
    ) {
        Ok(sc) => sc,
        Err(e) => match e {
            simul::SchedulerError::ArrivalRateTooSmall => panic!("arrivalRate should be positive"),
            simul::SchedulerError::ServiceRateTooSmall => {
                panic!("ServiceRate shouldn't be negative")
            }
        },
    };
    sc.submit(TaskDefinition::new(5, simul::task::Priority::High, Some(3)));
    match opt.weighted_dispatcher {
        true => {
            let mut sc = sc.with_weighted_dispatcher();
            run_simulation(&mut sc, opt.duration);
        }
        false => {
            run_simulation(&mut sc, opt.duration);
        }
    }
}
fn run_simulation<D>(sc: &mut Scheduler<D>, duration: usize)
where
    Scheduler<D>: JobDispatcher,
{
    for _ in 0..duration {
        // std::io::stdin().read_line(&mut String::new());
        // println!("{CLEAR}");
        sc.run();
        // println!("{sc}");
    }
    sc.export(&PathBuf::from("res.csv"));
}
