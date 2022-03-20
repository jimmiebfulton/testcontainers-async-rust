use crate::bollard::container::{InspectContainerOptions, RemoveContainerOptions};
use crate::bollard::Docker;
use async_trait::async_trait;
use std::collections::HashMap;

pub use crate::errors::TestcontainerError;
use crate::{DropAction, ImageSettings, Qualifier, Task};

const TESTCONTAINERS_DROP_ACTION: &str = "TESTCONTAINERS_DROP_ACTION";

#[async_trait]
pub trait Container: Sized {
    fn attach(handle: ContainerHandle, settings: ContainerSettings) -> Self;

    fn handle(&self) -> &ContainerHandle;

    fn handle_mut(&mut self) -> &mut ContainerHandle;

    fn settings(&self) -> &ContainerSettings;

    fn with_drop_action(mut self, drop_action: DropAction) -> Self {
        self.handle_mut().set_drop_action(drop_action);
        self
    }

    async fn host_port_for(&self, port: &str) -> Result<u16, TestcontainerError> {
        let result = self
            .handle()
            .docker
            .inspect_container(&self.handle().id, None::<InspectContainerOptions>)
            .await?;

        if let Some(network_settings) = result.network_settings {
            if let Some(port_map) = network_settings.ports {
                for pair in port_map.iter() {
                    if pair.0.starts_with(port) {
                        if let Some(bindings) = pair.1 {
                            if let Some(binding) = bindings.iter().next() {
                                return Ok(binding
                                    .host_port
                                    .as_ref()
                                    .map(|port| {
                                        port.parse::<u16>()
                                            .expect("Docker ports are expected to be u16")
                                    })
                                    .unwrap());
                            }
                        } else {
                            return Err(TestcontainerError::UnexposedPort {
                                portspec: port.to_owned(),
                            });
                        }
                    }
                }
            }
        }

        Err(TestcontainerError::UndefinedPort {
            portspec: port.to_owned(),
        })
    }

    async fn execute<T, R>(&self, task: T) -> Result<R, TestcontainerError>
    where
        T: Into<Box<dyn Task<Return = R> + 'static + Send + Sync>>,
        T: Send,
        R: 'static + Send + Sync,
    {
        let task = task.into();
        let result = task.execute(self.handle()).await?;
        Ok(result)
    }
}

pub struct ContainerHandle {
    id: String,
    docker: Docker,
    drop_action: DropAction,
}

impl ContainerHandle {
    pub fn new(id: String, docker: Docker) -> ContainerHandle {
        ContainerHandle {
            id,
            docker,
            drop_action: Default::default(),
        }
    }

    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    pub fn drop_action(&self) -> &DropAction {
        &self.drop_action
    }

    pub fn set_drop_action(&mut self, drop_action: DropAction) -> &Self {
        self.drop_action = drop_action;
        self
    }

    pub fn with_drop_action(mut self, drop_action: DropAction) -> Self {
        self.drop_action = drop_action;
        self
    }

    pub fn docker(&self) -> &Docker {
        &self.docker
    }
}

impl Drop for ContainerHandle {
    fn drop(&mut self) {
        let mut drop_action = self.drop_action.clone();

        if let Ok(value) = std::env::var(TESTCONTAINERS_DROP_ACTION) {
            match value.to_lowercase().as_str() {
                "remove" => drop_action = DropAction::Remove,
                "retain" => drop_action = DropAction::Retain,
                "stop" => drop_action = DropAction::Stop,
                value => eprintln!(
                    "'{}' is not a valid value for {}",
                    value, TESTCONTAINERS_DROP_ACTION
                ),
            }
        }

        match drop_action {
            DropAction::Remove => {
                let id = self.id.clone();
                let docker = self.docker.clone();
                let (sender, receiver) = std::sync::mpsc::channel();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        eprintln!("Removing container {id}");
                        let result = docker
                            .remove_container(
                                id.as_str(),
                                Some(RemoveContainerOptions {
                                    force: true,
                                    ..Default::default()
                                }),
                            )
                            .await;

                        match result {
                            Ok(_) => {}
                            Err(error) => {
                                eprintln!("Error removing container by id '{id}': {error}");
                            }
                        }

                        let _ = sender.send(());
                    });
                });
                let _ = receiver.recv();
            }
            DropAction::Retain => println!("Retaining container {}", self.id),
            DropAction::Stop => {
                let id = self.id.clone();
                let docker = self.docker.clone();
                let (sender, receiver) = std::sync::mpsc::channel();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        eprintln!("Stopping container {id}");
                        let result = docker.stop_container(id.as_str(), None).await;

                        match result {
                            Ok(_) => {}
                            Err(error) => {
                                eprintln!("Error stopping container by id '{id}': {error}");
                            }
                        }

                        let _ = sender.send(());
                    });
                });
                let _ = receiver.recv();
            }
        }
    }
}

pub struct ContainerSettings {
    name: String,
    qualifier: Qualifier,
    env: HashMap<String, Option<String>>,
}

impl ContainerSettings {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn fullname(&self) -> String {
        match &self.qualifier {
            Qualifier::Tag(tag) => format!("{}:{}", self.name, tag),
            Qualifier::Digest(digest) => format!("{}@{}", self.name, digest),
        }
    }

    pub fn qualifier(&self) -> &Qualifier {
        &self.qualifier
    }

    pub fn environment(&self) -> &HashMap<String, Option<String>> {
        &self.env
    }
}

impl From<&ImageSettings> for ContainerSettings {
    fn from(settings: &ImageSettings) -> Self {
        ContainerSettings {
            name: settings.name().to_owned(),
            qualifier: settings.qualifier().clone(),
            env: settings.environment().clone(),
        }
    }
}

#[async_trait]
pub trait ServiceContainer: Container {
    fn internal_service_port(&self) -> &str;

    async fn service_port(&self) -> Result<u16, TestcontainerError> {
        self.host_port_for(self.internal_service_port()).await
    }
}

#[async_trait]
pub trait AdminContainer: Container {
    fn internal_admin_port(&self) -> &str;

    async fn admin_port(&self) -> Result<u16, TestcontainerError> {
        self.host_port_for(self.internal_admin_port()).await
    }
}

#[async_trait]
pub trait DatabaseContainer: ServiceContainer {
    async fn protocol(&self) -> Result<&str, TestcontainerError>;

    async fn username(&self) -> Result<&str, TestcontainerError>;

    async fn password(&self) -> Result<&str, TestcontainerError>;

    async fn database(&self) -> Result<&str, TestcontainerError>;

    async fn jdbc_url(&self) -> Result<String, TestcontainerError>;

    async fn connect_cli(&self) -> Result<String, TestcontainerError>;

    async fn connect_url(&self) -> Result<String, TestcontainerError> {
        let username = self.username().await?;
        let password = self.password().await?;
        let protocol = self.protocol().await?;
        let port = self.service_port().await?;
        let database = self.database().await?;
        Ok(format!(
            "{protocol}://{username}:{password}@localhost:{port}/{database}"
        ))
    }
}
