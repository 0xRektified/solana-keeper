use anyhow::Result;

pub trait Executor<T> {
    fn execute(&self, state: &mut T) -> Result<()>;
}