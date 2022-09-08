pub mod project {
    tonic::include_proto!("projectmgt.v1alpha1");
}
use project::{projectmgt_client::ProjectmgtClient, GetProjectByRepositoryIdRequest};

use anyhow::{anyhow, Result};
use tonic::Request;

use crate::config::CONFIG;

#[allow(dead_code)]
pub async fn get_project_id(provider_id: String, repository_id: String) -> Result<u32> {
    info!("Getting project id.");
    let url: String = CONFIG
        .read()
        .grpc
        .get("project")
        .ok_or_else(|| anyhow!("Failed to find project url in config,"))?
        .to_owned();
    let mut client = ProjectmgtClient::connect(url).await?;
    let repository_id = repository_id.parse()?;
    let request = Request::new(GetProjectByRepositoryIdRequest {
        provider_id,
        repository_id,
    });
    let resp = client.get_project_by_repository_id(request).await?;
    let message = resp.into_inner();
    let project = message
        .project
        .ok_or_else(|| anyhow!("Project returned an empty message."))?;
    info!("Project_id recuperated!");
    debug!("project_id: {}", project.id);
    Ok(project.id as u32)
}
