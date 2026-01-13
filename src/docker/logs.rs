use crate::docker::client::DockerClient;
use bollard::container::LogsOptions;
use futures::Stream;
use bollard::container::LogOutput;

pub fn stream_logs(
    client: &DockerClient,
    container_id: &str,
    tail: &str,
) -> impl Stream<Item = Result<LogOutput, bollard::errors::Error>> {
    let options = LogsOptions::<String> {
        stdout: true,
        stderr: true,
        follow: true,
        tail: tail.to_string(),
        timestamps: true,
        ..Default::default()
    };
    
    client.inner.logs(container_id, Some(options))
}
