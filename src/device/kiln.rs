use std::{collections::{HashMap, VecDeque}};
use std::time::{Duration, SystemTime};
use std::sync::{Arc, Mutex};

use anyhow::Result;
use serde::Serialize;
use rsfuzzy::*;
use tokio::{task, time};
use tokio::sync::{mpsc, broadcast};
use tokio::time::sleep;
use tracing::{info, instrument, warn};
use uuid::Uuid;

use crate::schedule::NormalizedSchedule;
use crate::server::Command;
use crate::sensor::{Heater, MCP9600};

#[derive(Copy, Clone, Debug)]
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
    Start(NormalizedSchedule),
    Started,
    Stop,
    Stopped,
    Unchanged,
    Failure(String),
    Update
}

#[derive(Serialize)]
pub struct KilnUpdate {
    temperature: f64,
    timestamp: i64
}

///
#[derive(Debug)]
pub struct Kiln {
    pub id: Uuid,
    pub state: KilnState,
    thermocouple_address: u16, // 0x60
    heater_pin: u8,
    schedule: Option<NormalizedSchedule>,
    run_time: u64,
}

///
impl Kiln {
    #[instrument]
    pub fn start_schedule(&mut self, schedule: NormalizedSchedule) -> KilnState {
        info!("starting");
        match self.state {
            KilnState::Idle => {
                self.schedule = Some(schedule);
                self.state = KilnState::Running;
                self.run_time = 0;
            },
            KilnState::Running => {
                warn!("attempting to start schedule when one is already running");
            }
        }

        self.state
    }
    
    #[instrument]
    pub fn stop_schedule(&mut self) -> KilnState {
        info!("stopping");
        match self.state {
            KilnState::Idle => {
                warn!("attempting to stop schedule when kiln is already idle")
            },
            KilnState::Running => {
                self.schedule = None;
                self.state = KilnState::Idle;
                self.run_time = 0;
            },
        }

        self.state
    }

    #[instrument]
    pub async fn start(
        thermocouple_address: u16,
        heater_pin: u8,
        interval: u32,
        manager_sender: broadcast::Sender<Command>
    ) -> Result<mpsc::Sender<KilnEvent>> {
        info!("starting kiln");
        let channel = "kiln";
        let update_tx = manager_sender.clone();
        let queue = Arc::new(Mutex::new(VecDeque::<KilnEvent>::new()));
        let (tx, mut rx): (mpsc::Sender<KilnEvent>, mpsc::Receiver<KilnEvent>) = mpsc::channel(8);
        
        let update_queue = queue.clone();
        let _updater = task::spawn(async move {
            let mut wait_interval = time::interval(Duration::from_millis(interval as u64));
            let mut thermocouple = MCP9600::new(thermocouple_address).unwrap();
            let mut heater = Heater::new(heater_pin).unwrap();
            let mut runtime: u32 = 0;
            let mut schedule: Option<NormalizedSchedule> = None;
            let mut state = KilnState::Idle;
    
            loop {
                let temperature = &thermocouple.read().unwrap();
                let maybe_update = {
                    update_queue.lock().expect("unable to lock update queue").pop_front()
                };

                match maybe_update {
                    Some(KilnEvent::Start(s)) => {
                        state = KilnState::Running;
                        runtime = 0;
                        schedule = Some(s);
                    },
                    Some(KilnEvent::Stop) => {
                        state = KilnState::Idle;
                        runtime = 0;
                        schedule = None;
                    },
                    _ => (),
                };

                match state {
                    KilnState::Running => {
                        let set_point = schedule.clone().expect("valid").target_temperature(runtime);

                        heater.on();
                        sleep(Duration::from_millis((interval / 2) as u64)).await;
                        heater.off();
                        sleep(Duration::from_millis((interval / 2) as u64)).await;

                        let update = format!("{{ \"temperature\": {}, \"state\": \"{:?}\", \"set_point\": {} }}", temperature, state, set_point);
                        let _ = update_tx.send(Command::Update {
                            channel: channel.to_string(),
                            data: update.clone(),
                        });

                        info!("{}", update.clone());
                        runtime += interval / 1000;
                    },
                    KilnState::Idle => {
                        wait_interval.tick().await;
                        let _ = update_tx.send(Command::Update {
                            channel: channel.to_string(),
                            data: format!("{{ \"temperature\": {}, \"state\": \"{:?}\" }}", temperature, state),
                        });
                    },
                };
            }
        });

        let handler_queue = queue.clone();
        let _ = task::spawn(async move {
            while let Some(event) = rx.recv().await {
                info!("kiln got event");
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
    pub fn fuzzy() -> FuzzyController {
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
    k_i: f32,
    k_p: f32,
    k_d: f32,
    last_now: std::time::SystemTime,
    i_term: f32,
    last_error: f32,
}

///  A pretty blatant ripoff/rewrite of:
///    https://github.com/jbruce12000/kiln-controller/blob/master/lib/oven.py#L322
impl PID {
    fn init(k_i: f32, k_p: f32, k_d: f32) -> PID {
        PID {
            k_i: k_i,
            k_p: k_p,
            k_d: k_d,
            last_now: SystemTime::now(),
            i_term: 0.0,
            last_error: 0.0,
        }
    }

    fn compute(&mut self, set_point: f32, is_point: f32) -> f32 {
        let now = SystemTime::now();
        let delta: u64 = self.last_now.elapsed().unwrap().as_secs();

        let error: f32 = set_point - is_point;

        self.i_term += error * delta as f32 * self.k_i;
        let mut sorted = vec![-1.0, self.i_term, 1.0];
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        self.i_term = sorted[1];
    
        let d_error = (error - self.last_error) / delta as f32;

        let o: f32 = (self.k_p * error) + self.i_term + self.k_d * d_error;
        let mut sorted = vec![-1.0, o, 1.0];
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let output = sorted[1];

        self.last_error = error;
        self.last_now = now;
        output
    }
}
