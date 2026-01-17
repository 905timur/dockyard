use crate::docker::client::DockerClient;
use crate::types::{Result, AppError};
use std::process::Command;

pub async fn exec_interactive_shell(_client: &DockerClient, container_id: &str) -> Result<()> {
    // We use std::process::Command to leverage the 'docker' CLI which handles PTY/signals correctly
    // Try /bin/bash first
    let status = Command::new("docker")
        .arg("exec")
        .arg("-it")
        .arg(container_id)
        .arg("/bin/bash")
        .spawn()?
        .wait()?;

    if !status.success() {
        // Fallback to /bin/sh
        let status_sh = Command::new("docker")
            .arg("exec")
            .arg("-it")
            .arg(container_id)
            .arg("/bin/sh")
            .spawn()?
            .wait()?;
        
        if !status_sh.success() {
             return Err(AppError::Other("Failed to start shell (bash or sh) in container".to_string()));
        }
    }
    
    Ok(())
}
