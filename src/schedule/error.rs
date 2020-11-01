use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum ScheduleError {
    InvalidStep {
        description: String,
    },
    IOError { },
    InvalidYaml {
        location: String
    },
}

impl From<std::io::Error> for ScheduleError {
    fn from(_error: std::io::Error) -> ScheduleError {
        ScheduleError::IOError { }
    }
}

impl From<serde_yaml::Error> for ScheduleError {
    fn from(error: serde_yaml::Error) -> ScheduleError {
        ScheduleError::InvalidYaml {
            location: format!("{:?}", error.location()),
        }
    }
}