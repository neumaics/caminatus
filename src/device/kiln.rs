use std::{collections::{HashMap, VecDeque}};
use std::time::{Duration, SystemTime};
use std::sync::{Arc, Mutex};

use anyhow::Result;
use serde::Serialize;
use serde_json;
use rsfuzzy::*;
use tokio::task;
use tokio::sync::{mpsc, broadcast};
use tokio::time::sleep;
use tracing::{error, info, instrument, trace, warn};
use uuid::Uuid;

use crate::config::KilnConfig;
use crate::schedule::NormalizedSchedule;
use crate::server::Command;
use crate::sensor::{Heater, MCP9600};

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
    Start(NormalizedSchedule, ),
    Started,
    Stop,
    Stopped,
    Unchanged,
    Failure(String),
    Update
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
    set_point: f64
}

///
#[derive(Debug)]
pub struct Kiln {
    pub id: Uuid,
    pub state: KilnState,
    thermocouple_address: u16, // 0x60
    heater_pin: u8
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
                    update_queue.lock().expect("unable to lock update queue").pop_front()
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
                    },
                    Some(KilnEvent::Stop) | Some(KilnEvent::Complete)=> {
                        if state == KilnState::Idle {
                            warn!("attempting to stop already idle kiln");
                        }

                        state = KilnState::Idle;
                        runtime = 0;
                        schedule = None;
                    },
                    _ => (),
                };

                match state {
                    KilnState::Running => {
                        set_point = schedule.clone().expect("valid").target_temperature(runtime);
                        let p = pid.compute(&set_point, temperature);
                        let _f = FuzzyController::init().compute(0.0, 0.0); //ugh
                        let on_time = (interval as f64 * p).floor() as u64;
                        let off_time = (interval as u64) - on_time;

                        info!(on_time, off_time);
                        heater.on();
                        sleep(Duration::from_millis(on_time)).await;
                        heater.off();
                        sleep(Duration::from_millis(off_time)).await;

                        runtime += interval / 1000;

                        if runtime > schedule.clone().expect("valid").total_duration() {
                            info!("schedule complete, stopping kiln");
                            update_queue.lock().expect("unable to lock update queue").push_back(KilnEvent::Complete);
                        }
                    }
                    KilnState::Idle => {
                        sleep(Duration::from_millis((interval) as u64)).await;
                    },
                };

                let update = KilnUpdate {
                    runtime, state, set_point, temperature: *temperature,
                };
                let update = serde_json::to_string(&update).expect("expected valid kiln update serialization");

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
                    KilnEvent::Start(schedule) => handler_queue.lock().expect("unable to lock").push_back(KilnEvent::Start(schedule)),
                    KilnEvent::Stop => handler_queue.lock().expect("unable to lock").push_back(KilnEvent::Stop),
                    _ => ()
                }
            }
        });

        let _ = manager_sender.send(Command::Register { channel: channel.to_string() });

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
pub struct FuzzyController {
    engine: rsfuzzy::Engine
}

impl FuzzyController {
/// Ripped off from
///   https://github.com/auseckas/rsfuzzy
    pub fn init() -> FuzzyController {
        let mut f_engine = rsfuzzy::Engine::new();

        let i_var1 = fz_input_var![
            ("down", "normal", vec![0.0, 30.0]),
            ("triangle", "low", vec![15.0, 30.0, 40.0]),
            ("triangle", "medium", vec![30.0, 40.0, 55.0]),
            ("triangle", "high", vec![40.0, 60.0, 75.0]),
            ("up", "critical", vec![60.0, 100.0])
        ];
        f_engine.add_input_var("var1", i_var1, 0, 100);

        let i_var2 = fz_input_var![
            ("down", "normal", vec![0.0, 30.0]),
            ("triangle", "low", vec![15.0, 30.0, 40.0]),
            ("triangle", "medium", vec![30.0, 40.0, 55.0]),
            ("triangle", "high", vec![40.0, 60.0, 75.0]),
            ("up", "critical", vec![60.0, 100.0])
        ];

        f_engine.add_input_var("var2", i_var2, 0, 100);

        let o_var = fz_output_var![
            ("down", "normal", vec![0.0, 30.0]),
            ("triangle", "low", vec![15.0, 30.0, 40.0]),
            ("triangle", "medium", vec![30.0, 40.0, 55.0]),
            ("triangle", "high", vec![40.0, 60.0, 75.0]),
            ("up", "critical", vec![60.0, 100.0])
        ];
        f_engine.add_output_var("output", o_var, 0, 100);

        let f_rules = vec![
            ("if var1 is normal and var2 is normal then output is normal"),
            ("if var1 is very low and var2 is normal then output is very low"),
            ("if var1 is low then output is low"),
            ("if var1 is medium then output is medium"),
            ("if var1 is high then output is high"),
            ("if var1 is critical then output is critical"),
            ("if var1 is low and var2 is high then output is medium"),
        ];

        f_engine.add_rules(f_rules);
        f_engine.add_defuzz("centroid");

        FuzzyController {
            engine: f_engine
        }
    }

    pub fn compute(self, var1: f32, var2: f32) -> f32 {
        let inputs = fz_set_inputs![
            ("var1", var1),
            ("var2", var2)
        ];

        self.engine.calculate(inputs)
    }
}


#[derive(Debug)]
struct PID {
    k_i: f64,
    k_p: f64,
    k_d: f64,
    last_now: std::time::SystemTime,
    i_term: f64,
    last_error: f64,
}

///  A pretty blatant ripoff/rewrite of:
///    https://github.com/jbruce12000/kiln-controller/blob/master/lib/oven.py#L322
impl PID {
    fn init(k_i: f64, k_p: f64, k_d: f64) -> PID {
        PID {
            k_i,
            k_p,
            k_d,
            last_now: SystemTime::now(),
            i_term: 0.0,
            last_error: 0.0,
        }
    }

    fn compute(&mut self, set_point: &f64, is_point: &f64) -> f64 {
        let now = SystemTime::now();
        let delta: u64 = self.last_now.elapsed().unwrap().as_secs();

        let error: f64 = *set_point - *is_point;

        self.i_term += error * delta as f64 * self.k_i;
        let mut sorted = vec![-1.0, self.i_term, 1.0];
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        self.i_term = sorted[1];
    
        let d_error = (error - self.last_error) / delta as f64;

        let o: f64 = (self.k_p * error) + self.i_term + self.k_d * d_error;
        let mut sorted = vec![-1.0, o, 1.0];
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let output = sorted[1];

        self.last_error = error;
        self.last_now = now;
        output
    }
}
