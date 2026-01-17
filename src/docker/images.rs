use crate::docker::client::DockerClient;
use crate::types::{ImageInfo, Result, AppError};
use bollard::image::{ListImagesOptions, CreateImageOptions, RemoveImageOptions, PruneImagesOptions};
use bollard::models::ImageInspect;
use futures::stream::BoxStream;
use futures::StreamExt;
use futures::TryStreamExt;
use std::collections::HashMap;

pub async fn list_images(client: &DockerClient, show_dangling: bool) -> Result<Vec<ImageInfo>> {
    let mut filters = HashMap::new();
    if !show_dangling {
        filters.insert("dangling".to_string(), vec!["false".to_string()]);
    }
    
    let options = ListImagesOptions {
        filters,
        ..Default::default()
    };

    let images = client.inner.list_images(Some(options)).await?;

    let infos = images
        .into_iter()
        .map(|i| {
            // Clean up ID (remove sha256: prefix)
            let raw_id = i.id.replace("sha256:", "");
            let id = raw_id.chars().take(12).collect();
            
            ImageInfo {
                id,
                repo_tags: i.repo_tags,
                size: i.size,
                created: i.created,
            }
        })
        .collect();
    
    Ok(infos)
}

pub async fn inspect_image(client: &DockerClient, id: &str) -> Result<ImageInspect> {
    client.inner.inspect_image(id).await.map_err(Into::into)
}

pub async fn remove_image(client: &DockerClient, id: &str, force: bool) -> Result<()> {
    let options = RemoveImageOptions {
        force,
        ..Default::default()
    };
    // remove_image returns a list of changes, we can ignore it for now or return it if needed.
    // The requirement is just to return success/error.
    client.inner.remove_image(id, Some(options), None).await?;
    Ok(())
}

pub fn pull_image(client: &DockerClient, image: String) -> BoxStream<'static, Result<bollard::models::CreateImageInfo>> {
    let options = CreateImageOptions {
        from_image: image,
        ..Default::default()
    };
    
    // create_image returns impl Stream<Item = Result<CreateImageInfo, Error>>
    let stream = client.inner.create_image(Some(options), None, None);
    
    stream
        .map_err(AppError::Docker)
        .boxed()
}

pub async fn prune_images(client: &DockerClient) -> Result<()> {
     let mut filters = HashMap::new();
     filters.insert("dangling".to_string(), vec!["true".to_string()]);
     
     let options = PruneImagesOptions {
         filters,
     };
     client.inner.prune_images(Some(options)).await?;
     Ok(())
}
