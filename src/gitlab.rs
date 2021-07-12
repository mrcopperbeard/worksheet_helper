use std::error::Error;
use reqwest::{Client, StatusCode, header::HeaderMap};
use activity::{Activity, ActivityJson};
pub use activity::ActivityWeightGrid;
use serde::{Serialize, Deserialize};

use crate::TimeFilter;

mod activity;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub username: String
}

pub struct GitlabClient {
    client: Client
}

impl GitlabClient {
    pub fn new(private_token: String) -> Result<Self, Box<dyn Error>> {
        let mut headers = HeaderMap::new();
        let private_token = reqwest::header::HeaderValue::from_str(&private_token)?;
        headers.insert("PRIVATE-TOKEN", private_token);
    
        let client = reqwest::ClientBuilder::new()
            .default_headers(headers)
            .build()?;

        Ok(Self {
            client
        })
    }

    pub async fn get_user(&self) -> Result<User, Box<dyn Error>> {
        let response = self.client
            .get("http://git.esphere.local/api/v4/user")
            .send()
            .await?;

        let status_code = response.status();

        if status_code != StatusCode::OK {
            let error : Box<dyn Error> = format!("Bad status code: {}", status_code).into();

            return Err(error);
        }

        let user = response.json().await?;

        Ok(user)
    }

    pub async fn get_weight_grid(&self, filter: &TimeFilter, user_id: i64) -> Result<ActivityWeightGrid, Box<dyn Error>> {
        let request_uri = format!("http://git.esphere.local/api/v4/users/{}/events?after={}&before={}",
            user_id,
            filter.from.format("%Y-%m-%d"),
            filter.to.format("%Y-%m-%d"));

        let issues : Vec<Activity> = self.client
            .get(request_uri)
            .send()
            .await?
            .json::<Vec<ActivityJson>>()
            .await?
            .iter()
            .flat_map(|activity| activity.get_activities())
            .collect();
    
        let weight_grid = Activity::get_weigth_map(
            filter.to - filter.from,
            issues.iter());

        Ok(weight_grid)
    }
}