use serde::Serialize;

use super::Schedule;

#[derive(Debug, Serialize)]
pub enum ScheduleError {
    InvalidStep {
        description: String,
    },
    IOError {
        description: String
     },
    InvalidYaml {
        location: String
    },
    InvalidJson { },
}

impl From<std::io::Error> for ScheduleError {
    fn from(error: std::io::Error) -> ScheduleError {
        ScheduleError::IOError {
            description: format!("{:?}", error)
        }
    }
}

impl From<serde_yaml::Error> for ScheduleError {
    fn from(error: serde_yaml::Error) -> ScheduleError {
        ScheduleError::InvalidYaml {
            location: format!("{:?}", error.location()),
        }
    }
}

impl From<serde_json::Error> for ScheduleError {
    fn from(_error: serde_json::Error) -> ScheduleError {
        ScheduleError::InvalidJson { }
    }
}

// impl From<nom::Err> for ScheduleError {
//     fn from(error: nom::Err) -> ScheduleError {
//         ScheduleError::InvalidStep {
//             description: "foo".to_string()
//         }
//     }
// }

// impl From<nom::Err<nom::Err::Incomplete>> for Schedule {
//     fn from(error: nom::error::Err) -> ScheduleError {
//         ScheduleError::InvalidStep {
//             description: format!("{:?}", error)
//         }
//     }
// }
