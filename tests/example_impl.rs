use testcontainers_async::{async_trait, ContainerSettings, TestcontainerError};
use testcontainers_async::{Container, ContainerHandle, Image, ImageSettings};

pub struct ExampleImage {
    settings: ImageSettings,
}

impl Default for ExampleImage {
    fn default() -> Self {
        ExampleImage {
            settings: ImageSettings::new("redis", "latest"),
        }
    }
}

#[async_trait]
impl Image for ExampleImage {
    type ContainerType = ExampleContainer;

    fn settings(&self) -> &ImageSettings {
        &self.settings
    }

    fn settings_mut(&mut self) -> &mut ImageSettings {
        &mut self.settings
    }
}

#[derive(Debug)]
pub struct ExampleContainer {
    handle: ContainerHandle,
    settings: ContainerSettings,
}

impl ExampleContainer {
    pub async fn primary_port(&self) -> Result<u16, TestcontainerError> {
        self.host_port_for("6379").await
    }
}

#[async_trait]
impl Container for ExampleContainer {
    fn attach(handle: ContainerHandle, settings: ContainerSettings) -> Self {
        ExampleContainer { handle, settings }
    }

    fn handle(&self) -> &ContainerHandle {
        &self.handle
    }

    fn handle_mut(&mut self) -> &mut ContainerHandle {
        &mut self.handle
    }

    fn settings(&self) -> &ContainerSettings {
        &self.settings
    }
}
