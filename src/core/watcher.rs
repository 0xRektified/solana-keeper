use std::time::Duration;
use std::cell::RefCell;
use anyhow::Result;
use crate::core::trigger::{Trigger};
use crate::core::executor::{Executor};


pub struct Watcher<T> {
    pub trigger: Box<dyn Trigger<T>>,
    pub executor: Box<dyn Executor<T>>,
    pub duration: Duration,
}

impl<T: Clone> Watcher<T> {
     pub async fn run(&self, state: &RefCell<T>) -> Result<()> {
        loop {
            let mut current_state = state.borrow().clone();
            if self.trigger.should_trigger(&current_state)? {
                self.executor.execute(&mut current_state)?;
                // Write back the mutated state to RefCell
                *state.borrow_mut() = current_state;
            }
            tokio::time::sleep(self.duration).await;
        }
    }
}