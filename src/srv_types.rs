use reqwest::Error as ReqwestError;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Deserialize, Serialize, Debug)]
pub struct FileMetadata {
    pub path: String,
    pub hash: String,
    pub signature: String,
    pub file_size: i64,
    pub last_modified: String, // date-time
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DiffResponse {
    pub path: String,
    pub hash: String,
    pub diff: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ApplyDiffResponse {
    pub path: String,
    pub current_hash: String,
    pub previous_hash: String,
}

#[derive(Debug)]
pub enum SyftServerError {
    ServerError(String),
    ReqwestError(ReqwestError),
}

impl From<ReqwestError> for SyftServerError {
    fn from(err: ReqwestError) -> Self {
        Self::ReqwestError(err)
    }
}

impl fmt::Display for SyftServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReqwestError(err) => write!(f, "Reqwest Error: {}", err),
            Self::ServerError(s) => write!(f, "Server Error: {}", s),
        }
    }
}

impl std::error::Error for SyftServerError {}

#[derive(Debug)]
pub enum SyftAuthenticationError {}

#[derive(Debug)]
pub enum SyftNotFound {
    NotFoundError(String),
    ReqwestError(ReqwestError),
}

impl From<ReqwestError> for SyftNotFound {
    fn from(err: ReqwestError) -> Self {
        Self::ReqwestError(err)
    }
}

impl fmt::Display for SyftNotFound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReqwestError(err) => write!(f, "Reqwest Error: {}", err),
            Self::NotFoundError(s) => write!(f, "Not Found Error: {}", s),
        }
    }
}
