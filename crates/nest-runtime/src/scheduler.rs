//! Background task scheduler for Nest

use anyhow::Result;
use chrono::Utc;
use cron::Schedule;
use nest_api::scheduler::ScheduledTask;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Task scheduler
#[derive(Debug, Default)]
pub struct Scheduler {
    tasks: HashMap<String, ScheduledTask>,
}

impl Scheduler {
    /// Create new scheduler
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
        }
    }

    /// Schedule a new task
    pub fn schedule_task(&mut self, task: ScheduledTask) -> Result<()> {
        // Validate cron expression
        let _schedule = Schedule::try_from(&*task.schedule)?;

        // Calculate next run time
        let next_run = Self::calculate_next_run(&task.schedule)?;

        let mut task = task;
        task.next_run = next_run;

        self.tasks.insert(task.id.clone(), task);

        Ok(())
    }

    /// Remove a scheduled task
    pub fn unschedule_task(&mut self, task_id: &str) -> Option<ScheduledTask> {
        self.tasks.remove(task_id)
    }

    /// Get all scheduled tasks
    pub fn tasks(&self) -> &HashMap<String, ScheduledTask> {
        &self.tasks
    }

    /// Check for tasks that are due to run
    pub fn check_due_tasks(&mut self) -> Vec<ScheduledTask> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut due = Vec::new();
        let mut to_update = Vec::new();

        // First collect due tasks
        for (id, task) in &self.tasks {
            if task.enabled && task.next_run <= now {
                // Check if we can run this task again
                if task.max_runs > 0 && task.run_count >= task.max_runs {
                    continue;
                }

                due.push(task.clone());
                to_update.push(id.clone());
            }
        }

        // Now update their run counts and next run times
        for id in to_update {
            if let Some(task) = self.tasks.get_mut(&id) {
                task.run_count += 1;

                // Calculate next run time
                if let Ok(next_run) = Self::calculate_next_run(&task.schedule) {
                    task.next_run = next_run;
                } else {
                    task.enabled = false;
                }
            }
        }

        due
    }

    /// Calculate next run time from cron expression
    fn calculate_next_run(schedule: &str) -> Result<u64> {
        let schedule = Schedule::try_from(schedule)?;

        if let Some(next) = schedule.upcoming(Utc).next() {
            Ok(next.timestamp() as u64)
        } else {
            anyhow::bail!("No upcoming times for cron schedule")
        }
    }

    /// Get a specific task
    pub fn get_task(&self, task_id: &str) -> Option<&ScheduledTask> {
        self.tasks.get(task_id)
    }
}
