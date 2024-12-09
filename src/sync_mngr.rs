// SyncManager is a component responsible for synchronising
// data between local (federated node) and remote (federated server)
// the way data stored on node is dictated by StorageEngine trait
// From the high-level diagram:
//
// -----------------node----------------------      --server---
// Raw Data --> Function <--> Processed Data |<--> | Remote
//
// SyncManager is what handles Processed Data and Reciever.
// In theory, we should abstract over transport method, because
// one might prefer something else like infiBand protocol, inside
// a datacenter. As of now we focus only on how the processed data
// is stored and passed on to server and how we treat changes on
// server and what of that we bring to the node and how we store that.

use reqwest::Client;

use crate::endpoints::get_datasite_states;
use crate::srv_types::{FileMetadata, SyftServerError};
use std::collections::{HashMap, HashSet};

type State = HashMap<String, Vec<FileMetadata>>;

struct InMemoryStorage {
    state: State,
}

pub trait StorageEngine {
    fn get_state(&self) -> &State;
    fn update_state(&mut self, new: &State);
    fn delete_state(&mut self, state: &State);
}

impl StorageEngine for InMemoryStorage {
    fn get_state<'a>(&'a self) -> &'a State {
        &self.state
    }

    fn update_state(&mut self, new: &State) {
        for (key, values) in new {
            match self.state.get(key) {
                Some(other_values) => {
                    let other_set: HashSet<_> = other_values.iter().collect();
                    let diff: Vec<_> = values
                        .iter()
                        .filter(|v| !other_set.contains(v))
                        .cloned()
                        .collect();
                    if !diff.is_empty() {
                        self.state.insert(key.clone(), diff);
                    }
                }
                None => {
                    self.state.insert(key.clone(), values.clone());
                }
            }
        }
    }

    fn delete_state(&mut self, state: &State) {
        for (key, vec_to_remove) in state {
            if let Some(vec_in_map1) = self.state.get_mut(key) {
                vec_in_map1.retain(|item| !vec_to_remove.contains(item));
                if vec_in_map1.is_empty() {
                    self.state.remove(key);
                }
            }
        }
    }
}

struct SyncManager<S: StorageEngine> {
    client: reqwest::Client,
    email: String,
    state: S,
}

impl<S: StorageEngine> SyncManager<S> {
    async fn sync_datasites(
        &mut self,
        pull_remote: bool,
        push_local: bool,
    ) -> Result<(), SyftServerError> {
        let remote_state = get_datasite_states(&self.client, &self.email).await?;
        if pull_remote {
            let mut missing = diff_states(self.state.get_state(), &remote_state);
            // local state is the latest state
            missing.remove(&self.email);
            self.state.update_state(&missing);

            // remove files which were deleted remotely
            self.state
                .delete_state(&diff_states(&remote_state, self.state.get_state()));
        }
        if push_local {}
        Ok(())
    }
}

// Note: assumes that Vec<FileMetadata> is unique
fn diff_states(lhs: &State, rhs: &State) -> State {
    let mut res: State = HashMap::new();

    for (key, values) in lhs {
        match rhs.get(key) {
            Some(other_values) => {
                let other_set: HashSet<_> = other_values.iter().collect();
                let diff: Vec<_> = values
                    .iter()
                    .filter(|v| !other_set.contains(v))
                    .cloned()
                    .collect();
                if !diff.is_empty() {
                    res.insert(key.clone(), diff);
                }
            }
            None => {
                res.insert(key.clone(), values.clone());
            }
        }
    }

    res
}
