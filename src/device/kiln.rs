use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use anyhow::Result;
use serde::Serialize;
use rsfuzzy::*;
use tokio::{join, task, time};
use tokio::sync::mpsc;
use tokio::sync::broadcast::{Sender, Receiver};
use tokio::time::sleep;
use tracing::{info};
use uuid::Uuid;

use crate::{schedule::Schedule, server::Message};
use crate::server::Command;
use crate::sensor::{Heater, MCP9600};

#[derive(Copy, Clone, Debug)]
pub enum KilnState {
    Idle,
    Running,
}

// impl std::fmt::Display for KilnState {
//     fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {

//     }
// }

pub enum KilnEvent {
    Start,
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
pub struct Kiln {
    pub id: Uuid,
    pub state: KilnState,
    // pub sender: Sender<KilnEvent>,
    // receiver: Receiver<KilnEvent>,
    thermocouple_address: u16, // 0x60
    heater_pin: u8,
    schedule: Option<Schedule>,
}

///
impl Kiln {
    pub fn new(thermocouple_address: u16, heater_pin: u8) -> Result<Kiln> {
        let id = Uuid::new_v4();
        // let (tx, rx) = mpsc::channel(32);

        Ok(Kiln {
            id,
            state: KilnState::Idle,
            // sender: tx,
            // receiver: rx,
            thermocouple_address,
            heater_pin,
            schedule: None,
        })
    }

    fn update_state(current_state: KilnState, event: Option<KilnEvent>) -> KilnState {
        match event {
            Some(KilnEvent::Start) => Kiln::start_schedule(current_state),
            Some(KilnEvent::Stop) => Kiln::stop_schedule(current_state),
            _ => current_state,
        }
    }

    fn start_schedule(current_state: KilnState) -> KilnState {
        info!("starting");
       current_state
    }

    fn stop_schedule(current_state: KilnState) -> KilnState {
        info!("stopping");
        current_state
    }

    pub async fn start(self, interval: u32, manager_sender: Sender<Command>) -> Result<()> {
        let channel = "kiln";
        let update_tx = manager_sender.clone();

        let updater = task::spawn(async move {
            let mut wait_interval = time::interval(Duration::from_millis(interval as u64));
            let mut thermocouple = MCP9600::new(self.thermocouple_address).unwrap();
            let mut heater = Heater::new(self.heater_pin).unwrap();
            // let mut rx = self.receiver;
            let mut state = self.state;
            // let mut update: (Option<KilnEvent>) -> KilnEvent = Kiln::update_state;
    
            loop {
                let temperature = &thermocouple.read().unwrap();
                // let maybe_update = rx.recv().await;
                // let new_state = Kiln::update_state(state, maybe_update);

                // state = new_state;
                info!("current state {:?}", state);
                let new_state = KilnState::Idle;
                match new_state {
                    KilnState::Running => {
                        heater.on();
                        sleep(Duration::from_millis((interval / 2) as u64)).await;
                        heater.off();
                        sleep(Duration::from_millis((interval / 2) as u64)).await;
                        let _ = update_tx.send(Command::Update {
                            channel: channel.to_string(),
                            data: format!("{{ \"temperature\": {}, \"state\": \"{:?}\"}}", temperature, new_state),
                        });
                    },
                    KilnState::Idle => {
                        wait_interval.tick().await;
                        let _ = update_tx.send(Command::Update {
                            channel: channel.to_string(),
                            data: format!("{{ \"temperature\": {}, \"state\": \"{:?}\"}}", temperature, new_state),
                        });
                    },
                    
                };
            }
        });

        let _ = manager_sender.send(Command::Register { channel: channel.to_string() });
        let _ = join!(updater);

        Ok(())
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
