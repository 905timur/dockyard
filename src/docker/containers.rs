use crate::docker::client::DockerClient;
use crate::types::{ContainerInfo, Result};
use bollard::container::{ListContainersOptions, InspectContainerOptions, RemoveContainerOptions};
use bollard::models::ContainerInspectResponse;
use std::collections::HashMap;

pub async fn list_containers(client: &DockerClient, all: bool) -> Result<Vec<ContainerInfo>> {
    let mut filters = HashMap::new();
    if !all {
        filters.insert("status".to_string(), vec!["running".to_string()]);
    }

    let options = ListContainersOptions {
        all,
        filters,
        ..Default::default()
    };

    let containers = client.inner.list_containers(Some(options)).await?;

    let infos = containers
        .into_iter()
        .map(|c| {
            let state = c.state.as_deref().unwrap_or("unknown");
            
            let ports = c.ports.as_ref().map(|p| {
                 p.iter()
                    .take(2)
                    .filter_map(|port| {
                        if let Some(public) = port.public_port {
                            Some(format!("{}→{}", public, port.private_port))
                        } else {
                            Some(port.private_port.to_string())
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            }).unwrap_or_default();
            
            // Replicating logic from main.rs for short_id
            let short_id = c.id.as_ref()
                .map(|id| id.chars().take(12).collect())
                .unwrap_or_default();

            ContainerInfo {
                id: c.id.unwrap_or_default(),
                short_id,
                name: c.names.as_ref().and_then(|n| n.first()).map(|n| n.trim_start_matches('/').to_string()).unwrap_or_default(),
                status: c.status.unwrap_or_default(),
                image: c.image.unwrap_or_default(),
                ports,
                created: c.created.unwrap_or(0),
                state: state.to_string(),
            }
        })
        .collect();

    Ok(infos)
}

pub async fn inspect_container(client: &DockerClient, id: &str) -> Result<ContainerInspectResponse> {
    client.inner.inspect_container(id, None::<InspectContainerOptions>).await.map_err(Into::into)
}

pub async fn start_container(client: &DockerClient, id: &str) -> Result<()> {
    client.inner.start_container::<String>(id, None).await.map_err(Into::into)
}

pub async fn stop_container(client: &DockerClient, id: &str) -> Result<()> {
    client.inner.stop_container(id, None).await.map_err(Into::into)
}

pub async fn restart_container(client: &DockerClient, id: &str) -> Result<()> {
    client.inner.restart_container(id, None).await.map_err(Into::into)
}

pub async fn remove_container(client: &DockerClient, id: &str) -> Result<()> {
    let options = RemoveContainerOptions {
        force: true,
        ..Default::default()
    };
    client.inner.remove_container(id, Some(options)).await.map_err(Into::into)
}
