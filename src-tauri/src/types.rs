use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Segment {
    pub id: String,
    pub name: String,
    pub speaker: String,
    pub duration: u32,
    pub side: String,
    #[serde(rename = "type")]
    pub segment_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub segments: Vec<Segment>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TeamMembers {
    pub member1: String,
    pub member2: String,
    pub member3: String,
    pub member4: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TeamSettings {
    pub debate_topic: String,
    pub positive_team_name: String,
    pub negative_team_name: String,
    pub positive_members: TeamMembers,
    pub negative_members: TeamMembers,
}

impl Default for TeamSettings {
    fn default() -> Self {
        TeamSettings {
            debate_topic: String::new(),
            positive_team_name: String::new(),
            negative_team_name: String::new(),
            positive_members: TeamMembers {
                member1: String::new(),
                member2: String::new(),
                member3: String::new(),
                member4: String::new(),
            },
            negative_members: TeamMembers {
                member1: String::new(),
                member2: String::new(),
                member3: String::new(),
                member4: String::new(),
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimerRecord {
    pub id: String,
    pub template_id: String,
    pub template_name: String,
    pub debate_topic: String,
    pub positive_team_name: String,
    pub negative_team_name: String,
    pub start_time: String,
    pub end_time: String,
    pub segments: Vec<SegmentRecord>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SegmentRecord {
    pub segment_name: String,
    pub speaker: String,
    pub planned_duration: u32,
    pub actual_duration: u32,
}
