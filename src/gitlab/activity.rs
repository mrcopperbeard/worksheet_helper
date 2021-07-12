use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use lazy_static::lazy_static;
use regex::Regex;
use chrono::{DateTime, Local, Duration};

#[serde(rename_all = "lowercase")]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Copy, Clone)]
enum GitAction {
    Created,
    Deleted,
    Accepted,
    Opened,
    Approved,
    
    #[serde(rename="pushed to")]
    PushedTo,
    
    #[serde(rename="pushed new")]
    PushedNew,

    #[serde(rename="commented on")]
    CommentedOn,
}

#[derive(Debug, Serialize, Deserialize)]
struct PushData {
    #[serde(rename="ref")]
    reference: String,
    commit_title: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityJson {
    #[serde(rename="action_name")]
    action: GitAction,
    target_title: Option<String>,
    push_data: Option<PushData>,
    created_at: Option<DateTime<Local>>
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Activity {
    issue_name: String,
    datetime: Option<DateTime<Local>>,
    action: GitAction
}

#[derive(Debug)]
pub struct ActivityWeight {
    pub issue_name: String,
    pub weight: i32,
    pub duration: Duration,
}

#[derive(Debug)]
pub struct ActivityWeightGrid {
    pub total_weight: i32,
    pub total_duration: Duration,
    pub items: Vec<ActivityWeight>
}

impl ActivityJson {
    pub fn get_activities(&self) -> Vec<Activity> {
        let mut searching_fields : Vec<&str> = Vec::new();

        if let Some(target_title) = &self.target_title {
            searching_fields.push(target_title);
        }

        if let Some(push_data) = &self.push_data {
            searching_fields.push(&push_data.reference);

            if let Some(commit_title) = &push_data.commit_title {
                searching_fields.push(&commit_title);  
            }
        }

        let text = searching_fields.join("");

        lazy_static! {
            static ref RE : Regex = Regex::new(r"((?:SBINV|ITS)-\d+)").unwrap();
        }

        RE
            .captures_iter(&text)
            .map(|cap| Activity {
                issue_name: cap[1].into(),
                action: self.action,
                datetime: self.created_at
            })
            .collect()
    }
}

impl Activity {
    fn calculate_weight(&self) -> i32 {
        match self.action {
            GitAction::Created => 10,
            GitAction::PushedNew => 7,
            GitAction::PushedTo => 5,
            _ => 1,
        }
    }

    pub fn get_weigth_map<'a>(
        total_duration: Duration,
        activities: impl Iterator<Item=&'a Activity>)
        -> ActivityWeightGrid {
        let mut total_weight = 0;
        let mut map : HashMap<&str, i32> = HashMap::new();

        for activity in activities {
            let issue_name : &str = &activity.issue_name;
            let weight = activity.calculate_weight();
            total_weight += weight;
            if let Some(activity_weight) = map.get_mut(issue_name) {
                *activity_weight += weight;
            } else {
                map.insert(issue_name, weight);
            }
        }

        let items : Vec<ActivityWeight>  = map
            .iter()
            .map(|(issue_name, weight)| {
                let weight = *weight;
                let duration = total_duration * weight / total_weight;

                ActivityWeight {
                    weight,
                    duration,
                    issue_name: String::from(*issue_name),
                }
            })
            .collect();

        ActivityWeightGrid {
            total_weight,
            total_duration,
            items
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_activities_success_test() {
        // arrange
        let activity = ActivityJson {
            action: GitAction::CommentedOn,
            created_at: None,
            push_data: Some(PushData {
                reference: "SBINV-00001".into(),
                commit_title: Some("SBINV-00002: Add stuff".into()),
            }),
            target_title: Some("SBINV-23256: [Кодировка] ITS-100500 Не корректная кодировка pdf файлов в Расширенном экспорте".into()),
        };

        let expected: Vec<Activity> = vec![
            "SBINV-23256",
            "ITS-100500",
            "SBINV-00001",
            "SBINV-00002",
        ]
            .iter()
            .map(|issue| Activity {
                issue_name: String::from(*issue),
                action: GitAction::CommentedOn,
                datetime: None })
            .collect();

        // act
        let issues = activity.get_activities();

        // assert
        assert_eq!(issues, expected);
    }

    #[test]
    fn get_empty_activity_list_test() {
        // arrange
        let activity = ActivityJson {
            action: GitAction::CommentedOn,
            push_data: None,
            created_at: None,
            target_title: None,
        };

        // act
        let activities = activity.get_activities();

        // assert
        assert_eq!(activities.len(), 0);
    }

    #[test]
    fn get_weight_map_test() {
        // arrange
        let activities = vec![
            Activity { issue_name: "1".to_string(), action: GitAction::PushedNew, datetime: None },
            Activity { issue_name: "1".to_string(), action: GitAction::CommentedOn, datetime: None },
            Activity { issue_name: "2".to_string(), action: GitAction::CommentedOn, datetime: None },
        ];

        // act
        let weight_map = Activity::get_weigth_map(
            Duration::hours(1),
            activities.iter());

        // assert
        assert_eq!(weight_map.total_weight, 9);
        assert_eq!(weight_map.items[0].weight, 8);
        assert_eq!(weight_map.items[1].weight, 1);
    }
}