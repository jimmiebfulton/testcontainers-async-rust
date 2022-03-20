use crate::container::{ContainerSettings, DatabaseContainer};
use crate::tasks::MatchLogOutput;
use crate::{
    Container, ContainerHandle, Image, ImageSettings, ServiceContainer, TestcontainerError,
};
use async_trait::async_trait;

const IMAGE_NAME: &str = "postgres";
const DEFAULT_TAG: &str = "latest";
const POSTGRES_DB: &str = "POSTGRES_DB";
const POSTGRES_USER: &str = "POSTGRES_USER";
const POSTGRES_PASSWORD: &str = "POSTGRES_PASSWORD";
const POSTGRES_HOST_AUTH_METHOD: &str = "POSTGRES_HOST_AUTH_METHOD";

pub struct PostgresImage {
    settings: ImageSettings,
}

impl PostgresImage {
    pub fn with_database<T: Into<String>>(mut self, database: T) -> Self {
        self.settings_mut()
            .set_env_variable(POSTGRES_DB, Some(database));
        self
    }

    pub fn with_username<T: Into<String>>(mut self, username: T) -> Self {
        self.settings_mut()
            .set_env_variable(POSTGRES_USER, Some(username));
        self
    }

    pub fn with_password<T: Into<String>>(mut self, password: T) -> Self {
        self.settings_mut()
            .set_env_variable(POSTGRES_PASSWORD, Some(password));
        self.settings_mut()
            .set_env_variable(POSTGRES_HOST_AUTH_METHOD, None::<String>);
        self
    }
}

impl Default for PostgresImage {
    fn default() -> Self {
        PostgresImage {
            settings: ImageSettings::new(IMAGE_NAME, DEFAULT_TAG)
                .with_cmd(Some(vec!["postgres"]))
                .with_env_variable(POSTGRES_HOST_AUTH_METHOD.to_owned(), Some("trust"))
                // .with_task(MatchLogOutput::pattern("PostgreSQL init process complete; ready for start up."))
                .with_task(MatchLogOutput::containing_in_order(vec![
                    "PostgreSQL init process complete; ready for start up.",
                    "database system is ready to accept connections",
                ])),
        }
    }
}

#[async_trait]
impl Image for PostgresImage {
    type ContainerType = PostgresContainer;

    fn settings(&self) -> &ImageSettings {
        &self.settings
    }

    fn settings_mut(&mut self) -> &mut ImageSettings {
        &mut self.settings
    }
}

pub struct PostgresContainer {
    handle: ContainerHandle,
    settings: ContainerSettings,
}

#[async_trait]
impl Container for PostgresContainer {
    fn attach(handle: ContainerHandle, settings: ContainerSettings) -> Self {
        PostgresContainer { handle, settings }
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
impl ServiceContainer for PostgresContainer {
    fn internal_service_port(&self) -> &str {
        "5432/tcp"
    }
}

#[async_trait]
impl DatabaseContainer for PostgresContainer {
    async fn protocol(&self) -> Result<&str, TestcontainerError> {
        Ok("postgres")
    }

    async fn username(&self) -> Result<&str, TestcontainerError> {
        if let Some(Some(value)) = self.settings.environment().get(POSTGRES_USER) {
            return Ok(value);
        }
        Ok("postgres")
    }

    async fn password(&self) -> Result<&str, TestcontainerError> {
        if let Some(Some(value)) = self.settings.environment().get(POSTGRES_PASSWORD) {
            return Ok(value);
        }
        Ok("password")
    }

    async fn database(&self) -> Result<&str, TestcontainerError> {
        if let Some(Some(value)) = self.settings.environment().get(POSTGRES_DB) {
            return Ok(value);
        }
        Ok("postgres")
    }

    async fn jdbc_url(&self) -> Result<String, TestcontainerError> {
        let username = self.username().await?;
        let password = self.password().await?;
        let port = self.service_port().await?;
        let database = self.database().await?;
        Ok(format!(
            "jdbc:postgresql://localhost:{port}/{database}?user={username}&password={password}"
        ))
    }

    async fn connect_cli(&self) -> Result<String, TestcontainerError> {
        let username = self.username().await?;
        let port = self.service_port().await?;
        let database = self.database().await?;
        Ok(format!(
            "psql -U {username} -h localhost -p {port} {database}"
        ))
    }
}
