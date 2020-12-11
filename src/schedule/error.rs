use serde::Serialize;

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
    fn from(error: serde_json::Error) -> ScheduleError {
        ScheduleError::InvalidJson { }
    }
}