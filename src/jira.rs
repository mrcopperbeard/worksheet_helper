use std::error::Error;
use chrono::{DateTime, Utc};
use reqwest::{Client, StatusCode, header::HeaderMap, header::HeaderValue};
use base64::encode;
use serde::{Serialize, Deserialize};

use crate::{TimeFilter, time::WorkTime};

#[derive(Debug, Deserialize)]
pub struct JiraUser {
    pub key: String,
    pub name: String,
    #[serde(rename="displayName")]
    pub display_name: String,
}

#[derive(Debug, Deserialize)]
struct Project {
    key: String,
    name: String,
}

pub struct JiraClient {
    client: Client,
}

#[derive(Debug, Serialize)]
struct SearchRequest {
    jql: String,
    #[serde(rename="startAt")]
    skip: i32,
    #[serde(rename="maxResults")]
    take: i32,
}

#[derive(Debug, Deserialize)]
struct IssueFields {
    #[serde(rename="timespent")]
    time_spent: i64,
    summary: String,
    created: DateTime<Utc>,
    #[serde(rename="lastViewed")]
    last_viewed: DateTime<Utc>,
    project: Project,
    creator: JiraUser,
    reporter: JiraUser,
    assignee: JiraUser,
}

pub struct JiraIssue {
    pub key: String,
    pub summary: String,
    pub spent_time: WorkTime
}

#[derive(Debug, Deserialize)]
struct Issue {
    id: String,
    key: String,
    fields: IssueFields
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    issues: Vec<Issue>
}

impl JiraClient {
    pub fn new(login: String, password: String) -> Result<Self, Box<dyn Error>> {
        let token = format!("{}:{}", login, password);
        let token = format!("Basic {}", encode(token));
        let token = HeaderValue::from_str(&token)?;
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", token);
        headers.insert("Content-Type", HeaderValue::from_str("application/json")?);
    
        let client = reqwest::ClientBuilder::new()
            .default_headers(headers)
            .build()?;

        Ok(Self {
            client
        })
    }

    pub async fn search_issues(&self, filter: &TimeFilter) -> Result<Vec<JiraIssue>, Box<dyn Error>> {
        let jql = format!("worklogAuthor = currentUser() AND worklogDate > {} AND worklogDate < {}",
            filter.from.format("%Y-%m-%d"),
            filter.to.format("%Y-%m-%d"));

        let request = SearchRequest {
            jql,
            skip: 0,
            take: 2
        };

        let request = serde_json::to_string(&request)?;
        let response = self.client
            .post("https://jira.esphere.ru/rest/api/2/search")
            .body(request)
            .send()
            .await?;

        let status_code = response.status();

        if status_code != StatusCode::OK {
            let error : Box<dyn Error> = format!("Bad status code: {}", status_code).into();

            return Err(error);
        }

        let response : SearchResponse = response
            .json()
            .await?;

        let issues = response.issues
            .iter()
            .map(|issue| JiraIssue {
                key: issue.key.clone(),
                summary: issue.fields.summary.clone(),
                spent_time: WorkTime::from_seconds(issue.fields.time_spent)
            })
            .collect();

        Ok(issues)
    }
}