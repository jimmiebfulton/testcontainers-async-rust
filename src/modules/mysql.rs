use crate::container::ContainerSettings;
use crate::tasks::MatchLogOutput;
use crate::{Container, ContainerHandle, Image, ImageSettings, ServiceContainer};

const IMAGE_NAME: &str = "mysql";
const DEFAULT_TAG: &str = "latest";
const MYSQL_DATABASE: &str = "MYSQL_DATABASE";
const MYSQL_USER: &str = "MYSQL_USER";
const MYSQL_PASSWORD: &str = "MYSQL_PASSWORD";
const MYSQL_ALLOW_EMPTY_PASSWORD: &str = "MYSQL_ALLOW_EMPTY_PASSWORD";

pub struct MySqlImage {
    settings: ImageSettings,
}

impl Default for MySqlImage {
    fn default() -> Self {
        MySqlImage {
            settings: ImageSettings::new(IMAGE_NAME, DEFAULT_TAG)
                .with_env_variable(MYSQL_ALLOW_EMPTY_PASSWORD, Some("yes"))
                .with_task(MatchLogOutput::containing(
                    "/usr/sbin/mysqld: ready for connections",
                )),
        }
    }
}

impl MySqlImage {
    pub fn with_database<D: Into<String>>(mut self, database: D) -> Self {
        self.settings_mut()
            .set_env_variable(MYSQL_DATABASE, Some(database));
        self
    }

    pub fn with_username<U: Into<String>>(mut self, username: U) -> Self {
        self.settings_mut()
            .set_env_variable(MYSQL_USER, Some(username));
        self
    }

    pub fn with_password<P: Into<String>>(mut self, password: P) -> Self {
        self.settings_mut()
            .set_env_variable(MYSQL_PASSWORD, Some(password));
        self
    }
}

impl Image for MySqlImage {
    type ContainerType = MySqlContainer;

    fn settings(&self) -> &ImageSettings {
        &self.settings
    }

    fn settings_mut(&mut self) -> &mut ImageSettings {
        &mut self.settings
    }
}

pub struct MySqlContainer {
    handle: ContainerHandle,
    settings: ContainerSettings,
}

impl Container for MySqlContainer {
    fn attach(handle: ContainerHandle, settings: ContainerSettings) -> Self {
        MySqlContainer { handle, settings }
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

impl ServiceContainer for MySqlContainer {
    fn internal_service_port(&self) -> &str {
        "3306/tcp"
    }
}
