use async_trait::async_trait;

use crate::container::ContainerSettings;
use crate::tasks::MatchLogOutput;
use crate::{Container, ContainerHandle, Image, ImageSettings, ServiceContainer};

const IMAGE_NAME: &str = "redis";
const DEFAULT_TAG: &str = "latest";

pub struct RedisImage {
    settings: ImageSettings,
}

impl Default for RedisImage {
    fn default() -> Self {
        RedisImage {
            settings: ImageSettings::new(IMAGE_NAME, DEFAULT_TAG)
                .with_task(MatchLogOutput::containing("Ready to accept connections")),
        }
    }
}

#[async_trait]
impl Image for RedisImage {
    type ContainerType = RedisContainer;

    fn settings(&self) -> &ImageSettings {
        &self.settings
    }

    fn settings_mut(&mut self) -> &mut ImageSettings {
        &mut self.settings
    }
}

pub struct RedisContainer {
    handle: ContainerHandle,
    settings: ContainerSettings,
}

impl Container for RedisContainer {
    fn attach(handle: ContainerHandle, settings: ContainerSettings) -> Self {
        RedisContainer { handle, settings }
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

#[async_trait]
impl ServiceContainer for RedisContainer {
    fn internal_service_port(&self) -> &str {
        "6379/tcp"
    }
}
