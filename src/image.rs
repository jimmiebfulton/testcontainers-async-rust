use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use futures::TryStreamExt;

use crate::bollard::container::Config;
use crate::bollard::image::CreateImageOptions;
use crate::bollard::models::HostConfig;
use crate::bollard::Docker;
use crate::task::Task;
use crate::{async_trait, Container, ContainerHandle, TestcontainerError};

#[derive(Clone, Debug)]
pub enum DropAction {
    Remove,
    Retain,
    Stop,
}

impl Default for DropAction {
    fn default() -> Self {
        DropAction::Remove
    }
}

pub struct ImageSettings {
    name: String,
    qualifier: Qualifier,
    cmd: Option<Vec<String>>,
    entrypoint: Option<Vec<String>>,
    env: HashMap<String, Option<String>>,
    tasks: Vec<Box<dyn Task<Return = ()> + 'static + Send + Sync>>,
}

impl ImageSettings {
    pub fn new<N: Into<String>, Q: Into<Qualifier>>(name: N, qualifier: Q) -> ImageSettings {
        ImageSettings {
            name: name.into(),
            qualifier: qualifier.into(),
            cmd: Default::default(),
            entrypoint: Default::default(),
            env: Default::default(),
            tasks: Default::default(),
        }
    }

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

    pub fn set_qualifier<Q: Into<Qualifier>>(&mut self, qualifier: Q) -> &mut Self {
        self.qualifier = qualifier.into();
        self
    }

    pub fn with_qualifier<Q: Into<Qualifier>>(mut self, qualifier: Q) -> Self {
        self.set_qualifier(qualifier);
        self
    }

    pub fn cmd(&self) -> Option<&Vec<String>> {
        self.cmd.as_ref()
    }

    pub fn with_cmd<I, V>(mut self, cmd: Option<I>) -> ImageSettings
    where
        I: IntoIterator<Item = V>,
        V: Into<String>,
    {
        self.cmd = cmd.map(|cmd| cmd.into_iter().map(Into::into).collect());
        self
    }

    pub fn entrypoint(&self) -> Option<&Vec<String>> {
        self.entrypoint.as_ref()
    }

    pub fn environment(&self) -> &HashMap<String, Option<String>> {
        &self.env
    }

    pub fn with_entrypoint<I, V>(mut self, entrypoint: Option<I>) -> ImageSettings
    where
        I: IntoIterator<Item = V>,
        V: Into<String>,
    {
        self.entrypoint = entrypoint.map(|ep| ep.into_iter().map(Into::into).collect());
        self
    }

    pub fn set_env_variable<K: Into<String>, V: Into<String>>(
        &mut self,
        key: K,
        value: Option<V>,
    ) -> &mut ImageSettings {
        self.env.insert(key.into(), value.map(|v| v.into()));
        self
    }

    pub fn with_env_variable<K: Into<String>, V: Into<String>>(
        mut self,
        key: K,
        value: Option<V>,
    ) -> ImageSettings {
        self.set_env_variable(key.into(), value.map(|v| v.into()));
        self
    }

    pub fn tasks(&self) -> &Vec<Box<dyn Task<Return = ()> + 'static + Send + Sync>> {
        &self.tasks
    }

    pub fn append_task<T>(&mut self, task: T) -> &mut ImageSettings
    where
        T: Into<Box<dyn Task<Return = ()> + 'static + Send + Sync>>,
    {
        self.tasks.push(task.into());
        self
    }

    pub fn with_task<T>(mut self, task: T) -> ImageSettings
    where
        T: Into<Box<dyn Task<Return = ()> + 'static + Send + Sync>>,
    {
        self.append_task(task);
        self
    }
}

#[derive(Clone, Debug)]
pub enum Qualifier {
    Tag(String),
    Digest(String),
}

impl Display for Qualifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Qualifier::Tag(value) => write!(f, "{}", value),
            Qualifier::Digest(value) => write!(f, "{}", value),
        }
    }
}

impl Qualifier {
    pub fn tag<T: Into<String>>(tag: T) -> Qualifier {
        Qualifier::Tag(tag.into())
    }

    pub fn digest<D: Into<String>>(digest: D) -> Qualifier {
        Qualifier::Digest(digest.into())
    }
}

impl From<&str> for Qualifier {
    fn from(value: &str) -> Self {
        if value.starts_with("sha256") {
            Qualifier::digest(value)
        } else {
            Qualifier::tag(value)
        }
    }
}

impl From<String> for Qualifier {
    fn from(value: String) -> Self {
        value.as_str().into()
    }
}

#[async_trait]
pub trait Image: Sized {
    type ContainerType: Container;

    fn settings(&self) -> &ImageSettings;

    fn settings_mut(&mut self) -> &mut ImageSettings;

    fn set_env_variable<K: Into<String>, V: Into<String>>(
        &mut self,
        key: K,
        value: Option<V>,
    ) -> &mut Self {
        self.settings_mut().set_env_variable(key, value);
        self
    }

    fn with_env_variable<K: Into<String>, V: Into<String>>(
        mut self,
        key: K,
        value: Option<V>,
    ) -> Self {
        self.set_env_variable(key, value);
        self
    }

    fn set_entrypoint<I, T>(&mut self, entrypoint: I) -> &mut Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        self.settings_mut().entrypoint = Some(entrypoint.into_iter().map(Into::into).collect());
        self
    }

    fn with_entrypoint<I, T>(mut self, entrypoint: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        self.set_entrypoint(entrypoint);
        self
    }

    fn with_qualifier<Q: Into<Qualifier>>(mut self, qualifier: Q) -> Self {
        self.settings_mut().set_qualifier(qualifier);
        self
    }

    fn with_task<T>(mut self, task: T) -> Self
    where
        T: Into<Box<dyn Task<Return = ()> + 'static + Send + Sync>>,
    {
        self.settings_mut().append_task(task);
        self
    }

    async fn on_before_start_container(&self, _: &Docker) -> Result<(), TestcontainerError> {
        Ok(())
    }

    async fn on_after_start_container(
        &self,
        _: &ContainerHandle,
    ) -> Result<(), TestcontainerError> {
        Ok(())
    }

    async fn on_pull_image(&self, docker: &Docker) -> Result<(), TestcontainerError> {
        let inspect_result = docker
            .inspect_image(self.settings().fullname().as_str())
            .await;

        // TODO: Implement PullPolicy?
        match inspect_result {
            Ok(_) => (),
            Err(_) => {
                eprintln!("Pulling image {}", self.settings().fullname());
                docker
                    .create_image(
                        Some(CreateImageOptions {
                            from_image: self.settings().fullname(),
                            ..Default::default()
                        }),
                        None,
                        None,
                    )
                    .try_collect::<Vec<_>>()
                    .await?;
            }
        }

        Ok(())
    }

    async fn on_create_container(
        &self,
        docker: Docker,
    ) -> Result<ContainerHandle, TestcontainerError> {
        let host_config = Some(HostConfig {
            publish_all_ports: Some(true),

            ..Default::default()
        });

        let env: Vec<String> = self
            .settings()
            .environment()
            .iter()
            .map(|(k, v)| {
                if let Some(v) = v {
                    format!("{k}={v}")
                } else {
                    k.to_owned()
                }
            })
            .collect();

        let image_config = Config {
            image: Some(self.settings().fullname()),
            host_config,
            cmd: self.settings().cmd().cloned(),
            entrypoint: self.settings().entrypoint().cloned(),
            env: Some(env),
            tty: Some(true),
            ..Default::default()
        };

        let id = docker
            .create_container::<&str, String>(None, image_config)
            .await?
            .id;
        Ok(ContainerHandle::new(id, docker))
    }

    async fn on_start_container(&self, handle: &ContainerHandle) -> Result<(), TestcontainerError> {
        handle
            .docker()
            .start_container::<String>(handle.id(), None)
            .await?;

        Ok(())
    }

    async fn on_execute_tasks(&self, handle: &ContainerHandle) -> Result<(), TestcontainerError> {
        for task in self.settings().tasks() {
            task.execute(handle).await?;
        }
        Ok(())
    }

    async fn start_container(&self) -> Result<Self::ContainerType, TestcontainerError> {
        let docker = Docker::connect_with_local_defaults()?;
        self.start_container_with_docker(docker).await
    }

    async fn start_container_with_docker(
        &self,
        docker: Docker,
    ) -> Result<Self::ContainerType, TestcontainerError> {
        self.on_pull_image(&docker).await?;
        self.on_before_start_container(&docker).await?;
        let handle = self.on_create_container(docker).await?;
        self.on_start_container(&handle).await?;
        self.on_after_start_container(&handle).await?;
        self.on_execute_tasks(&handle).await?;
        Ok(Self::ContainerType::attach(handle, self.settings().into()))
    }
}
