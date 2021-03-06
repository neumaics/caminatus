use std::collections::HashMap;
use std::time::SystemTime;

use rsfuzzy::*;
pub struct Fuzzy {
    engine: rsfuzzy::Engine,
}

impl Fuzzy {
    /// Ripped off from
    ///   https://github.com/auseckas/rsfuzzy
    pub fn init(step_size: f32) -> Fuzzy {
        let mut engine = rsfuzzy::Engine::new();
        let high_bound = 2.0 * step_size;
        let low_bound = -high_bound;

        let error = fz_input_var![
            ("down", "negative", vec![low_bound, 0.0]),
            ("triangle", "low", vec![-step_size, 0.0, step_size]),
            ("up", "positive", vec![0.0, high_bound])
        ];
        engine.add_input_var("error", error, low_bound as usize, high_bound as usize);

        let output = fz_output_var![
            ("down", "heat", vec![0.0, 50.0]),
            ("triangle", "hold", vec![25.0, 0.0, 75.0]),
            ("up", "cool", vec![50.0, 100.0])
        ];
        engine.add_output_var("output", output, 0, 1);

        let rules = vec![
            ("if error is negative then output is heat"),
            ("if error is low then output is hold"),
            ("if error is positive then output is cool")
        ];

        engine.add_rules(rules);
        engine.add_defuzz("centroid");

        Fuzzy { engine }
    }

    /// todo: add derivative input variable
    pub fn compute(self, error: f32) -> f32 {
        let inputs = fz_set_inputs![("error", error)];

        self.engine.calculate(inputs)
    }
}

#[derive(Debug)]
pub struct PID {
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
    pub fn init(k_i: f64, k_p: f64, k_d: f64) -> PID {
        PID {
            k_i,
            k_p,
            k_d,
            last_now: SystemTime::now(),
            i_term: 0.0,
            last_error: 0.0,
        }
    }

    pub fn compute(&mut self, set_point: &f64, is_point: &f64) -> f64 {
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
