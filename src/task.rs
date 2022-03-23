use crate::{async_trait, ContainerHandle, TestcontainerError};
use std::fmt::Debug;

#[async_trait]
pub trait Task: 'static + Send + Sync + Debug {
    type Return;

    async fn execute(&self, handle: &ContainerHandle) -> Result<Self::Return, TestcontainerError>;
}

impl<T, R> From<T> for Box<dyn Task<Return = R> + 'static + Send + Sync>
where
    T: Task<Return = R> + 'static + Send + Sync,
{
    fn from(task: T) -> Self {
        Box::new(task)
    }
}
