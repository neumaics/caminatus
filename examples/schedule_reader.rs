use caminatus::schedule::Schedule;

fn main() {
    let schedule = Schedule::from_file("./schedules/sample.yaml".to_string());

    println!("{:?}", schedule);

    let normalized = schedule.unwrap().normalize();

    println!("{:?}", normalized);
}