use crate::container::ContainerSettings;
use crate::{Container, ContainerHandle, Image, ImageSettings, Qualifier};
use async_trait::async_trait;

pub struct GenericImage {
    settings: ImageSettings,
}

impl GenericImage {
    pub fn new<N: Into<String>, Q: Into<Qualifier>>(name: N, qualifier: Q) -> GenericImage {
        GenericImage {
            settings: ImageSettings::new(name, qualifier),
        }
    }
}

impl Image for GenericImage {
    type ContainerType = GenericContainer;

    fn settings(&self) -> &ImageSettings {
        &self.settings
    }

    fn settings_mut(&mut self) -> &mut ImageSettings {
        &mut self.settings
    }
}

pub struct GenericContainer {
    handle: ContainerHandle,
    settings: ContainerSettings,
}

#[async_trait]
impl Container for GenericContainer {
    fn attach(handle: ContainerHandle, settings: ContainerSettings) -> Self {
        GenericContainer { handle, settings }
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
