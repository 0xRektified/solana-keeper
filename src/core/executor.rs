use anyhow::Result;

pub trait Executor<T> {
    fn execute(&self, state: &T) -> Result<()>;
}