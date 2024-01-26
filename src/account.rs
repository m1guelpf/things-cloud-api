use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    StatusCode,
};

use crate::{tasks, History};

/// User-Agent header set by Things for macOS v3.13.8 (31308504)
const THINGS_USER_AGENT: &str = "ThingsMac/31516502";

#[derive(Debug)]
pub struct Account {
    pub email: String,
    pub maildrop_email: String,
    pub(crate) history_key: String,
    pub(crate) client: reqwest::Client,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The provided credentials are invalid
    #[error("The provided credentials are invalid")]
    InvalidCredentials,

    /// The account cannot be used with Things Cloud
    #[error("The account has issues: {0:?}")]
    AccountHasIssues(Vec<serde_json::Value>),

    /// The account has an unknown status
    #[error("The account has an unknown status: {0}")]
    UnknownAccountStatus(String),

    /// An error occurred while communicating with the Things Cloud API
    #[error("An error occurred while communicating with the Things Cloud API: {0}")]
    Reqwest(#[from] reqwest::Error),
}

impl Account {
    /// Log in to Things Cloud API with the provided credentials.
    ///
    /// # Errors
    ///
    /// Returns [`Error::AccountHasIssues`] if the account has outstanding issues.
    /// Returns [`Error::UnknownAccountStatus`] if the account has an unknown status.
    /// Returns [`Error::InvalidCredentials`] if the provided credentials are invalid.
    /// Returns [`Error::Reqwest`] if an error occurred while communicating with the Things Cloud API.
    pub async fn login(email: &str, password: &str) -> Result<Self, Error> {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            HeaderValue::from_static(THINGS_USER_AGENT),
        );
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Password {password}"))
                .map_err(|_| Error::InvalidCredentials)?,
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        let response = client
            .get(format!(
                "https://cloud.culturedcode.com/version/1/account/{email}"
            ))
            .header("Authorization", format!("Password {password}"))
            .send()
            .await?;

        if response.status() == StatusCode::UNAUTHORIZED {
            return Err(Error::InvalidCredentials);
        }

        let response = response.json::<AccountAPIResponse>().await?;

        if !response.issues.is_empty() {
            return Err(Error::AccountHasIssues(response.issues));
        }

        if response.status != Status::Active {
            return Err(Error::InvalidCredentials);
        }

        Ok(Self {
            client,
            email: response.email,
            history_key: response.history_key,
            maildrop_email: response.maildrop_email,
        })
    }

    /// Fetch the list of tasks from the Things Cloud API.
    ///
    /// # Errors
    ///
    /// Returns [`tasks::Error`] if an error occurred while fetching the tasks.
    pub async fn history(&self) -> Result<History, tasks::Error> {
        History::from_account(self).await
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
struct AccountAPIResponse {
    status: Status,
    #[serde(rename = "SLA-version-accepted")]
    sla_version: String,
    email: String,
    history_key: String,
    maildrop_email: String,
    issues: Vec<serde_json::Value>,
}

#[derive(Debug, serde::Deserialize, PartialEq, Eq)]
enum Status {
    #[serde(rename = "SYAccountStatusActive")]
    Active,
    Other(String),
}
