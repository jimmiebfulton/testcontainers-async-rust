use crate::container::ContainerSettings;
use crate::tasks::MatchLogOutput;
use crate::{AdminContainer, Container, ContainerHandle, Image, ImageSettings, ServiceContainer};
use async_trait::async_trait;

const IMAGE_NAME: &str = "cockroachdb/cockroach";
const DEFAULT_TAG: &str = "latest";

pub struct CockroachDbImage {
    settings: ImageSettings,
}

impl Default for CockroachDbImage {
    fn default() -> Self {
        CockroachDbImage {
            settings: ImageSettings::new(IMAGE_NAME, DEFAULT_TAG)
                .with_cmd(Some(vec![
                    "start-single-node",
                    "--insecure",
                    "--accept-sql-without-tls",
                ]))
                .with_task(MatchLogOutput::containing("nodeID:")),
        }
    }
}

#[derive(Debug)]
pub struct CockroachDbContainer {
    handle: ContainerHandle,
    settings: ContainerSettings,
}

impl Container for CockroachDbContainer {
    fn attach(handle: ContainerHandle, settings: ContainerSettings) -> Self {
        CockroachDbContainer { handle, settings }
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
impl Image for CockroachDbImage {
    type ContainerType = CockroachDbContainer;

    fn settings(&self) -> &ImageSettings {
        &self.settings
    }

    fn settings_mut(&mut self) -> &mut ImageSettings {
        &mut self.settings
    }
}

#[async_trait]
impl ServiceContainer for CockroachDbContainer {
    fn internal_service_port(&self) -> &str {
        "26257/tcp"
    }
}

#[async_trait]
impl AdminContainer for CockroachDbContainer {
    fn internal_admin_port(&self) -> &str {
        "8080/tcp"
    }
}
