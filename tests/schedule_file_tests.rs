use caminatus::schedule::Schedule;

#[test]
fn works() {
    let schedule = Schedule::from_file("./tests/sample_schedules/valid.yaml".to_string());

    assert!(schedule.is_ok());
}

#[test]
fn rejects_schedule_with_too_few_steps() {
    let filename = "./tests/sample_schedules/not_enough_steps.yaml";
    let schedule = Schedule::from_file(filename.to_string());

    assert!(schedule.is_err());
}

#[test]
fn rejects_schedule_with_invalid_steps() {
    let filename = "./tests/sample_schedules/duration_and_rate.yaml";
    let schedule = Schedule::from_file(filename.to_string());
    assert!(schedule.is_err());

    let filename = "./tests/sample_schedules/no_change_in_step.yaml";
    let schedule = Schedule::from_file(filename.to_string());
    assert!(schedule.is_err());
}

// TODO: test normalization
