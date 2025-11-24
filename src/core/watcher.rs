use std::time::Duration;
use anyhow::Result;
use crate::core::trigger::{Trigger};
use crate::core::executor::{Executor};

pub struct Watcher<T> {
    pub trigger: Box<dyn Trigger<T>>,
    pub executor: Box<dyn Executor<T>>,
    pub duration: Duration,
}

impl<T> Watcher<T> {
     async fn run<F>(&self, mut fetch_state: F) -> Result<()>
     where
        F: FnMut() -> Result<T>,
     {
        loop {
            let state:T = fetch_state()?; 
            if self.trigger.should_trigger(&state){
                self.executor.execute(&state)?;
            }
            tokio::time::sleep(self.duration).await;
        }
    }
}