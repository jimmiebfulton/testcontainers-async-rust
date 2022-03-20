#[derive(thiserror::Error, Debug)]
pub enum TestcontainerError {
    #[error("Error: {message}")]
    Generic { message: String },
    #[error("Internal port {portspec} is not exposed.")]
    UnexposedPort { portspec: String },
    #[error("Request port {portspec} is not defined for this image.")]
    UndefinedPort { portspec: String },
    #[error("Docker Error")]
    DockerError {
        #[from]
        source: bollard::errors::Error,
    },
}
