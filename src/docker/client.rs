use bollard::Docker;
use crate::types::{Result, AppError};

#[derive(Clone)]
pub struct DockerClient {
    pub(crate) inner: Docker,
}

impl DockerClient {
    pub fn new() -> Result<Self> {
        let inner = Docker::connect_with_local_defaults()
            .map_err(AppError::Docker)?;
        Ok(Self { inner })
    }
}
