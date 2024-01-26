use async_recursion::async_recursion;
use serde_json::{Map, Value};
use std::collections::HashMap;

use crate::Account;

#[derive(Debug)]
pub struct History<'a> {
    pub tasks: Vec<Value>,
    last_index: usize,
    account: &'a Account,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An error occurred while communicating with the Things Cloud API
    #[error("An error occurred while communicating with the Things Cloud API: {0}")]
    Reqwest(#[from] reqwest::Error),

    /// An error occurred while decoding the API response
    #[error("An error occurred while decoding the response: {0}")]
    Decode(#[from] serde_path_to_error::Error<serde_json::Error>),
}

impl<'a> History<'a> {
    pub(crate) async fn from_account(account: &'a Account) -> Result<Self, Error> {
        let history = Self::fetch_history(account, 0).await?;
        let tasks_history = history
            .items
            .into_iter()
            .map(|c| {
                c.into_iter()
                    .filter(|(_, item)| item.kind.starts_with("Task"))
            })
            .fold(HashMap::<String, Vec<_>>::new(), |mut acc, map| {
                for (key, value) in map {
                    acc.entry(key).or_default().push(value);
                }
                acc
            });

        let tasks = tasks_history
            .into_iter()
            .filter(|(_, task_history)| {
                !task_history
                    .iter()
                    .any(|item| item.action == ItemAction::Deleted)
            })
            .map(|(_, task_history)| {
                task_history
                    .into_iter()
                    .fold(Map::default(), |mut acc, mut item| {
                        let task = std::mem::take(item.payload.as_object_mut().unwrap());
                        acc.extend(task);
                        acc
                    })
            })
            .map(Value::Object)
            .collect::<Vec<_>>();

        Ok({
            Self {
                tasks,
                account,
                last_index: history.last_index,
            }
        })
    }

    #[must_use]
    pub const fn from_index(account: &'a Account, index: usize) -> Self {
        Self {
            account,
            tasks: vec![],
            last_index: index,
        }
    }

    #[async_recursion]
    async fn fetch_history(account: &Account, index: usize) -> Result<HistoryApiResponse, Error> {
        let request = account
            .client
            .get(format!(
                "https://cloud.culturedcode.com/version/1/history/{}/items?start-index={index}",
                account.history_key
            ))
            .send()
            .await?
            .bytes()
            .await?;

        let mut history = serde_path_to_error::deserialize::<_, HistoryApiResponse>(
            &mut serde_json::Deserializer::from_slice(&request),
        )?;

        if index + history.items.len() < history.last_index {
            let rec_history = Self::fetch_history(account, index + history.items.len()).await?;
            history.items.extend(rec_history.items);
        }

        Ok(history)
    }
}

#[derive(Debug, serde::Deserialize)]
struct HistoryApiResponse {
    #[serde(rename = "current-item-index")]
    last_index: usize,
    items: Vec<HashMap<String, HistoryItem>>,
}

#[derive(Debug, serde::Deserialize)]
struct HistoryItem {
    #[serde(rename = "e")]
    kind: String,
    #[serde(rename = "t")]
    action: ItemAction,
    #[serde(rename = "p")]
    payload: serde_json::Value,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, serde_repr::Deserialize_repr, PartialEq, Eq)]
enum ItemAction {
    Create,
    Modified,
    Deleted,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, serde_repr::Deserialize_repr, PartialEq, Eq)]
pub enum TaskStatus {
    Pending = 0,
    Completed = 3,
    Cancelled = 2,
}
