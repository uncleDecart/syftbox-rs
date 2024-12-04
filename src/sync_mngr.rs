use reqwest::Client;

use crate::endpoints::get_datasite_states;
use crate::srv_types::{FileMetadata, SyftServerError};
use std::collections::HashMap;

type State = HashMap<String, Vec<FileMetadata>>;

struct InMemoryStorage {
    state: State,
}

pub trait StorageEngine {
    fn get_state_diff(&self, rhs: &State) -> State;
    fn update_state(&self, new: &State);
    fn delete_state(&self, state: &State);
}

impl StorageEngine for InMemoryStorage {
    fn get_state_diff(&self, rhs: &State) -> State {
        return HashMap::new();
    }

    fn update_state(&self, new: &State) {}

    fn delete_state(&self, state: &State) {}
}

struct SyncManager<S: StorageEngine> {
    client: reqwest::Client,
    email: String,
    state: S,
}

impl SyncManager {
    async fn sync_datasites(
        &self,
        pull_remote: bool,
        push_local: bool,
    ) -> Result<(), SyftServerError> {
        let remote_state = get_datasite_states(&self.client, &self.email).await?;
        if pull_remote {
            self.download_missing(&remote_state).await?;
        }
        if push_local {
            self.upload_new(&remote_state).await?;
        }
        Ok(())
    }

    async fn download_missing(
        &self,
        datasites: &HashMap<String, Vec<FileMetadata>>,
    ) -> Result<(), SyftServerError> {
        Ok(())
    }

    async fn upload_new(
        &self,
        datasites: &HashMap<String, Vec<FileMetadata>>,
    ) -> Result<(), SyftServerError> {
        Ok(())
    }
}
