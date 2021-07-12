use std::error::Error;

use chrono::{Date, Duration, Utc};
use jira::JiraClient;
use time::WorkTime;
use gitlab::{ActivityWeightGrid, GitlabClient};

mod time;
mod gitlab;
mod jira;

pub struct TimeFilter {
    from: Date<Utc>,
    to: Date<Utc>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let today = Utc::today();
    let filter = TimeFilter {
        from: today - Duration::days(7),
        to: today + Duration::days(1),
    };

    get_gitlab_activity(&filter).await?;
    get_jira_activity(&filter).await?;

    Ok(())
}

async fn get_jira_activity(filter: &TimeFilter) -> Result<(), Box<dyn std::error::Error>> {
    println!("Collecting JIRA info...");

    let jira_login = std::env::var("JIRA_LOGIN")?;
    let jira_password = std::env::var("JIRA_PASSWORD")?;
    let jira_client = JiraClient::new(jira_login, jira_password)?;
    let issues = jira_client.search_issues(filter).await?;
    
    for issue in issues {
        println!("{}: {}, spent time: {}", &issue.key, &issue.summary, &issue.spent_time);
    }

    Ok(())
}

async fn get_gitlab_activity(filter: &TimeFilter) -> Result<(), Box<dyn Error>> {
    println!("Collecting Gitlab info...");

    let gitlab_token = std::env::var("GITLAB_TOKEN")?;
    let gitlab_client = GitlabClient::new(gitlab_token)?;
    let user = gitlab_client.get_user().await?;

    println!("User: {}", user.name);

    let weight_grid = gitlab_client
        .get_weight_grid(filter, user.id)
        .await?;

    print_weight_grid(&weight_grid);

    Ok(())
}

fn print_weight_grid(weight_grid: &ActivityWeightGrid) {
    for weight_info in weight_grid.items.iter() {
        let percentage = weight_info.weight as f32 / weight_grid.total_weight as f32  * 100.0;
        let time = WorkTime::new(weight_info.duration);

        println!(
            "{}: {:.2}% of time ({})",
            weight_info.issue_name,
            percentage,
            time
        );
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Local};

    #[test]
    fn duration_percentage_test() {
        let duration = Local::today().and_hms(18, 0, 0) - Local::today().and_hms(9, 0, 0);
        let duration : Duration = duration * 2 / 9;

        assert_eq!(duration.num_hours(), 2);
        assert_eq!(duration.num_minutes(), 120);
        assert_eq!(duration.num_seconds(), 0);
    }
}