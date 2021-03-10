use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;

use serde::Serialize;
use serde_json;
use tokio::sync::{broadcast, mpsc};
use tokio::task;
use tokio::time::sleep;
use tracing::{error, info, instrument, trace, warn};

mod controller;

use crate::config::KilnConfig;
use crate::schedule::NormalizedSchedule;
use crate::sensor::{Heater, MCP9600};
use crate::server::Command;
use controller::{Fuzzy, PID};

#[derive(Copy, Clone, Debug, PartialEq, Serialize)]
pub enum KilnState {
    Idle,
    Running,
}

#[derive(Debug)]
pub struct RunState {
    runtime: u32,
    schedule: Option<NormalizedSchedule>,
    state: KilnState,
}

#[derive(Debug)]
pub enum KilnEvent {
    Complete,
    Start(NormalizedSchedule),
    Started,
    Stop,
    Stopped,
    Unchanged,
    Failure(String),
    Update,
}

/// State of the kiln, sent to clients where
/// temperature and set_point: recorded temperature in C
/// runtime: time the schedule has been running in seconds
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KilnUpdate {
    temperature: f64,
    state: KilnState,
    runtime: u32,
    set_point: f64,
}

///
#[derive(Debug)]
pub struct Kiln {
    pub state: KilnState,
    thermocouple_address: u16, // 0x60
    heater_pin: u8,
}

///
impl Kiln {
    #[instrument]
    pub async fn start(
        thermocouple_address: u16,
        heater_pin: u8,
        interval: u32,
        manager_sender: broadcast::Sender<Command>,
        config: KilnConfig,
    ) -> Result<mpsc::Sender<KilnEvent>> {
        info!("starting kiln");
        let channel = "kiln";
        let update_tx = manager_sender.clone();
        let queue = Arc::new(Mutex::new(VecDeque::<KilnEvent>::new()));
        let (tx, mut rx): (mpsc::Sender<KilnEvent>, mpsc::Receiver<KilnEvent>) = mpsc::channel(8);

        let update_queue = queue.clone();
        let _updater = task::spawn(async move {
            let mut thermocouple = MCP9600::new(thermocouple_address).unwrap();
            let mut heater = Heater::new(heater_pin).unwrap();
            let mut runtime: u32 = 0;
            let mut schedule: Option<NormalizedSchedule> = None;
            let mut state = KilnState::Idle;
            let mut pid = PID::init(config.integral, config.proportional, config.derivative);

            loop {
                let temperature = &thermocouple.read().unwrap();
                let mut set_point: f64 = 0.0;
                let maybe_update = {
                    update_queue
                        .lock()
                        .expect("unable to lock update queue")
                        .pop_front()
                };

                match maybe_update {
                    Some(KilnEvent::Start(s)) => {
                        if state == KilnState::Running {
                            error!("attempting to start a schedule while a schedule is already running");
                        } else {
                            info!(name = s.name.as_str(), message = "starting schedule");
                            state = KilnState::Running;
                            runtime = 0;
                            schedule = Some(s);
                        }
                    }
                    Some(KilnEvent::Stop) | Some(KilnEvent::Complete) => {
                        if state == KilnState::Idle {
                            warn!("attempting to stop already idle kiln");
                        }

                        state = KilnState::Idle;
                        runtime = 0;
                        schedule = None;
                    }
                    _ => (),
                };

                match state {
                    KilnState::Running => {
                        set_point = schedule.clone().expect("valid").target_temperature(runtime);
                        let error = set_point - temperature;
                        let p = pid.compute(&set_point, temperature);
                        let f = Fuzzy::init(config.fuzzy_step_size).compute(error as f32); //ugh
                        let on_time = (interval as f64 * p).floor() as u64;
                        let off_time = (interval as u64) - on_time;

                        info!(on_time, off_time, "p: {} f: {}", p as f64, f as f64);
                        heater.on();
                        sleep(Duration::from_millis(on_time)).await;
                        heater.off();
                        sleep(Duration::from_millis(off_time)).await;

                        runtime += interval / 1000;

                        if runtime > schedule.clone().expect("valid").total_duration() {
                            info!("schedule complete, stopping kiln");
                            update_queue
                                .lock()
                                .expect("unable to lock update queue")
                                .push_back(KilnEvent::Complete);
                        }
                    }
                    KilnState::Idle => {
                        sleep(Duration::from_millis((interval) as u64)).await;
                    }
                };

                let update = KilnUpdate {
                    runtime,
                    state,
                    set_point,
                    temperature: *temperature,
                };
                let update = serde_json::to_string(&update)
                    .expect("expected valid kiln update serialization");

                trace!("{}", &update);

                let _ = update_tx.send(Command::Update {
                    channel: channel.to_string(),
                    data: update,
                });
            }
        });

        let handler_queue = queue.clone();
        let _ = task::spawn(async move {
            while let Some(event) = rx.recv().await {
                trace!("kiln got event");
                match event {
                    KilnEvent::Start(schedule) => handler_queue
                        .lock()
                        .expect("unable to lock")
                        .push_back(KilnEvent::Start(schedule)),
                    KilnEvent::Stop => handler_queue
                        .lock()
                        .expect("unable to lock")
                        .push_back(KilnEvent::Stop),
                    _ => (),
                }
            }
        });

        let _ = manager_sender.send(Command::Register {
            channel: channel.to_string(),
        });

        Ok(tx)
    }
}

#[derive(Debug)]
pub struct KilnError {}
impl std::error::Error for KilnError {}

impl std::fmt::Display for KilnError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Kiln Error")
    }
}
