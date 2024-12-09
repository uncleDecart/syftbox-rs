use crate::srv_types::*;
use base64::{engine::general_purpose, Engine};
use reqwest::{Client, Error, Response};
use serde_json;
use std::{collections::HashMap, fs::File, path::Path};

pub async fn handle_json_response(
    endpoint: &str,
    response: Response,
) -> Result<serde_json::Value, SyftServerError> {
    if response.status().is_success() {
        return response.json().await.map_err(|_| {
            SyftServerError::ServerError(format!("[{endpoint}] failed to deserialize JSON"))
        });
    }
    Err(SyftServerError::ServerError(format!(
        "[{endpoint}] call failed: {}",
        response.text().await.unwrap_or_default()
    )))
}

pub async fn get_access_token(client: &Client, email: &str) -> Result<String, Error> {
    let response = client
        .post("/auth/request_email_token")
        .json(&serde_json::json!({ "email": email }))
        .send()
        .await?;

    let email_token: serde_json::Value = response.json().await?;
    let email_token_str = email_token["email_token"]
        .as_str()
        .unwrap_or_default()
        .to_string();

    let response = client
        .post("/auth/validate_email_token")
        .header("Authorization", format!("Bearer {}", email_token_str))
        .send()
        .await?;

    let access_token: serde_json::Value = response.json().await?;
    Ok(access_token["access_token"]
        .as_str()
        .unwrap_or_default()
        .to_string())
}

pub async fn get_datasite_states(
    client: &Client,
    _email: &str,
) -> Result<HashMap<String, Vec<FileMetadata>>, SyftServerError> {
    let response = client.post("/sync/datasite_states").send().await?;
    let data = handle_json_response("/sync/datasite_states", response).await?;
    let mut result = HashMap::new();

    if let Some(metadata_list) = data.as_object() {
        for (email_key, items) in metadata_list {
            let metadata: Vec<FileMetadata> = serde_json::from_value(items.clone()).unwrap();
            result.insert(email_key.clone(), metadata);
        }
    }
    Ok(result)
}

pub async fn get_remote_state(
    client: &Client,
    path: &Path,
) -> Result<Vec<FileMetadata>, SyftServerError> {
    let response = client
        .post("/sync/dir_state")
        .json(&serde_json::json!({ "dir": path.to_str().unwrap() }))
        .send()
        .await?;

    let response_data = handle_json_response("/sync/dir_state", response).await?;
    let metadata_list: Vec<FileMetadata> = serde_json::from_value(response_data).unwrap();

    Ok(metadata_list)
}

pub async fn get_metadata(client: &Client, path: &Path) -> Result<FileMetadata, SyftServerError> {
    let response = client
        .post("/sync/get_metadata")
        .json(&serde_json::json!({ "path_like": path.to_str().unwrap() }))
        .send()
        .await?;

    let response_data = handle_json_response("/sync/get_metadata", response).await?;
    let metadata: FileMetadata = serde_json::from_value(response_data).unwrap();

    Ok(metadata)
}

pub async fn get_diff(
    client: &Client,
    path: &Path,
    signature: &[u8],
) -> Result<DiffResponse, SyftServerError> {
    let response = client
        .post("/sync/get_diff")
        .json(&serde_json::json!({
            "path": path.to_str().unwrap(),
            "signature": general_purpose::STANDARD.encode(signature),
        }))
        .send()
        .await?;

    let response_data = handle_json_response("/sync/get_diff", response).await?;
    let diff_response: DiffResponse = serde_json::from_value(response_data).unwrap();

    Ok(diff_response)
}

pub async fn apply_diff(
    client: &Client,
    path: &Path,
    diff: &[u8],
    expected_hash: &str,
) -> Result<ApplyDiffResponse, SyftServerError> {
    let response = client
        .post("/sync/apply_diff")
        .json(&serde_json::json!({
            "path": path.to_str().unwrap(),
            "diff": general_purpose::STANDARD.encode(diff),
            "expected_hash": expected_hash,
        }))
        .send()
        .await?;

    let response_data = handle_json_response("/sync/apply_diff", response).await?;
    let apply_diff_response: ApplyDiffResponse = serde_json::from_value(response_data).unwrap();

    Ok(apply_diff_response)
}

pub async fn delete(client: &Client, path: &Path) -> Result<(), SyftServerError> {
    let response = client
        .post("/sync/delete")
        .json(&serde_json::json!({ "path": path.to_str().unwrap() }))
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(SyftServerError::ServerError(format!(
            "Failed to delete file at {}",
            path.display()
        )));
    }

    Ok(())
}

pub async fn create(client: &Client, path: &Path, data: &[u8]) -> Result<(), SyftServerError> {
    let response = client
        .post("/sync/create")
        .multipart(
            reqwest::multipart::Form::new()
                .file("file", path)
                .await
                .unwrap(),
        )
        .send()
        .await?;

    handle_json_response("/sync/create", response).await?;
    Ok(())
}

pub async fn download(client: &Client, path: &Path) -> Result<Vec<u8>, SyftNotFound> {
    let response = client
        .post("/sync/download")
        .json(&serde_json::json!({ "path": path.to_str().unwrap() }))
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(SyftNotFound::NotFoundError(format!(
            "File not found on server: {}",
            path.display()
        )));
    }

    Ok(response.bytes().await.unwrap().to_vec())
}

pub async fn download_bulk(
    client: &Client,
    paths: Vec<String>,
) -> Result<Vec<u8>, SyftServerError> {
    let response = client
        .post("/sync/download_bulk")
        .json(&serde_json::json!({ "paths": paths }))
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(SyftServerError::ServerError(
            "Failed to download bulk files".to_string(),
        ));
    }

    Ok(response.bytes().await.unwrap().to_vec())
}

pub async fn whoami(client: &Client) -> Result<String, SyftServerError> {
    let response = client.post("/auth/whoami").send().await?;

    if response.status().is_success() {
        let json_response: serde_json::Value = response.json().await?;
        Ok(json_response["email"]
            .as_str()
            .unwrap_or_default()
            .to_string())
    } else if response.status().as_u16() == 401 {
        Err(SyftServerError::ServerError("Unauthorized".to_string()))
    } else {
        Err(SyftServerError::ServerError(format!(
            "Health check failed with status: {}",
            response.status()
        )))
    }
}
