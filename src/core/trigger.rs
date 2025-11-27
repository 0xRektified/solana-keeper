pub trait Trigger<T> {
    fn should_trigger(&self, state: &T) -> Result<bool, anyhow::Error>;
}