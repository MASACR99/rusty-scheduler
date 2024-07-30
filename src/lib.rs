use core::time;
use std::{collections::HashMap, sync::{Arc, Mutex}, thread};
use uuid::Uuid;
use chrono::{prelude::*, Duration, TimeDelta};
use rusty_cron::Cron;

#[derive(Clone)]
struct Task {
    task_id: Uuid,
    time_to_exec: DateTime<Utc>,
    cron: String,
    function_to_exec: fn(),
    execution_threshold_millis: Option<u64>,
}
impl Task{
    pub fn new(task_id: Uuid, time_to_exec: DateTime<Utc>, cron: String, function_to_exec: fn(), execution_threshold_millis: Option<u64>) -> Task{
        Task {task_id,  time_to_exec, cron, function_to_exec, execution_threshold_millis}
    }
}

// This struct allows for the configuration of the scheduler,
// it's optional to the creation of a scheduler and will
// use threshold 250 and wait 1000 by default
pub struct SchedulerConfigOptions {
    scheduler_wait_millis: u64,
    execution_threshold_millis: u64,
}
impl SchedulerConfigOptions {
    pub fn new(execution_threshold_millis: u64, scheduler_wait_millis: u64) -> SchedulerConfigOptions{
        SchedulerConfigOptions{execution_threshold_millis,scheduler_wait_millis }
    }
}

pub struct Scheduler{
    tasks: Arc<Mutex<Vec<Task>>>,
    config: Arc<SchedulerConfigOptions>
}
impl Scheduler{
    pub fn new(config: Option<SchedulerConfigOptions>) -> Scheduler{
        let config_options: Arc<SchedulerConfigOptions>;
        match config{
            Some(n) => config_options = Arc::new(n),
            None => config_options = Arc::new(SchedulerConfigOptions::new(250,1000))
        }
        
        let scheduler: Scheduler = Scheduler {tasks: Arc::new(Mutex::new(Vec::new())), config: config_options};
        return scheduler;
    }

    pub fn startup(&self){
        let tasks: Arc<Mutex<Vec<Task>>> = self.tasks.clone();
        let config: Arc<SchedulerConfigOptions> = self.config.clone();
        std::thread::spawn(move || {
            let mut running_tasks: HashMap<Uuid, thread::JoinHandle<()>> = HashMap::new();
            loop {
                let now: DateTime<Utc> = chrono::Utc::now();

                // To avoid concurrency problems we lock the mutex that protects the tasks array
                let mut guard = (&*tasks).lock().unwrap();
                for task in &mut *guard {
                    // If current task is on running map and has finished, clear from map and get next exec
                    if running_tasks.contains_key(&task.task_id) && running_tasks.get(&task.task_id).unwrap().is_finished() {
                        running_tasks.remove(&task.task_id);
                        let cloned_now = now.clone();
                        //Old time to exec +1 second to avoid executing the same time frame, instead cron will return next execution
                        let old_time_exec = task.time_to_exec.checked_add_signed(TimeDelta::seconds(1)); 
                        let time = cloned_now.checked_add_signed(Duration::milliseconds(Cron::parse_time(&task.cron, old_time_exec).unwrap())).unwrap();
                        task.time_to_exec = time;
                    }
                    
                    let task_threshold: u64;

                    match task.execution_threshold_millis {
                        Some(n) => task_threshold = n,
                        None => task_threshold = config.execution_threshold_millis
                    }

                    // If either execution should've happened in the past or execution is in the next few millis
                    // execute function of task
                    // Not "else if" because when the task has ended we want to know if it needs to execute now or within threshold
                    if !running_tasks.contains_key(&task.task_id) && 
                        ((task.time_to_exec - now).num_milliseconds() < task_threshold.try_into().unwrap())  
                    {
                        let task_to_run = task.clone();
                        let join_handle = thread::spawn(move || {
                            (task_to_run.function_to_exec)();
                        });

                        running_tasks.insert(task.task_id, join_handle);
                    }
                }

                thread::sleep(time::Duration::from_millis(config.scheduler_wait_millis));
            }
        });
    }

    pub fn add_task(&mut self, cron: &str, function_to_exec: fn(), execution_threshold_millis: Option<u64>) -> Result<Uuid,String>{

        let time_result = Cron::parse_time(cron, None);

        let time_to_exec: i64;
        match time_result {
            Ok(n) => time_to_exec = n,
            Err(e) => return Err(e)
        }
        let uuid = Uuid::new_v4();

        let now: DateTime<Utc> = chrono::Utc::now();
        let time_to_exec = now.checked_add_signed(Duration::milliseconds(time_to_exec)).unwrap();
        let guard = &mut *self.tasks.lock().unwrap();
        guard.push(Task::new(uuid, time_to_exec, cron.to_owned(), function_to_exec, execution_threshold_millis));

        return Ok(uuid);
    }

    pub fn remove_task(&mut self, task_id: Uuid){
        let guard = &mut *self.tasks.lock().unwrap();
        guard.retain(|task| task.task_id != task_id);
    }
}