use futures::StreamExt;
use std::collections::HashMap;

use crate::async_trait;
use crate::bollard::container::LogsOptions;
use crate::bollard::exec::{CreateExecOptions, StartExecResults};
use crate::task::Task;
use crate::{ContainerHandle, TestcontainerError};

#[derive(Debug)]
pub struct MatchLogOutput {
    patterns: Vec<String>,
}

impl MatchLogOutput {
    pub fn containing<P: Into<String>>(pattern: P) -> MatchLogOutput {
        MatchLogOutput {
            patterns: vec![pattern.into()],
        }
    }

    pub fn containing_in_order<I, P>(patterns: I) -> MatchLogOutput
    where
        I: IntoIterator<Item = P>,
        P: Into<String>,
    {
        MatchLogOutput {
            patterns: patterns.into_iter().map(Into::into).collect(),
        }
    }
}

#[async_trait]
impl Task for MatchLogOutput {
    type Return = ();

    async fn execute(&self, handle: &ContainerHandle) -> Result<Self::Return, TestcontainerError> {
        let log_options = Some(LogsOptions {
            follow: true,
            stdout: true,
            stderr: true,
            ..Default::default()
        });

        let mut logstream = handle.docker().logs::<String>(handle.id(), log_options);

        let mut remaining_patterns = self.patterns.iter();
        let mut current_pattern = remaining_patterns.next();

        while let Some(output) = logstream.next().await {
            if let Ok(line) = output {
                if let Some(pattern) = current_pattern {
                    if line.to_string().contains(pattern) {
                        current_pattern = remaining_patterns.next();
                        if current_pattern.is_none() {
                            break;
                        }
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Execute {
    cmd: Vec<String>,
    env: HashMap<String, Option<String>>,
    required_status: Option<u64>,
}

impl Execute {
    pub fn command<I, T>(command: I) -> Execute
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        Execute {
            cmd: command.into_iter().map(Into::into).collect(),
            env: Default::default(),
            required_status: None,
        }
    }

    pub fn with_env_variable<K: Into<String>, V: Into<String>>(
        mut self,
        key: K,
        value: Option<V>,
    ) -> Self {
        self.env.insert(key.into(), value.map(Into::into));
        self
    }

    pub fn with_required_status(mut self, required_status: u64) -> Execute {
        self.required_status = Some(required_status);
        self
    }
}

#[async_trait]
impl Task for Execute {
    type Return = ();

    async fn execute(&self, handle: &ContainerHandle) -> Result<Self::Return, TestcontainerError> {
        let env: Vec<String> = self
            .env
            .iter()
            .map(|(k, v)| {
                if let Some(v) = v {
                    format!("{k}={v}")
                } else {
                    k.to_owned()
                }
            })
            .collect();

        let exec = handle
            .docker()
            .create_exec(
                handle.id(),
                CreateExecOptions {
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    cmd: Some(self.cmd.clone()),
                    env: Some(env),
                    ..Default::default()
                },
            )
            .await?
            .id;
        if let StartExecResults::Attached { mut output, .. } =
            handle.docker().start_exec(&exec, None).await?
        {
            while let Some(Ok(msg)) = output.next().await {
                print!("{msg}");
            }
        }
        Ok(())
    }
}
