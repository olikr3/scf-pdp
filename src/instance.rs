use std::f64;
use std::fs::File;

#[derive(Debug, Clone, Copy)]
struct Point {
    x: f64,
    y: f64
}
#[derive(Debug)]
pub struct Instance {

    n_reqs: usize,
    n_vehicles: usize,
    cap: usize,
    gamma: usize, // min num of requests
    rho: usize, // fairness weight
    demands: Vec<usize>,
    locations: Vec<Point>,
}


impl Instance {
    
    fn from_ascii(filename: &str) -> Result<Self> {
        let file = File::open(filename)?;
    }


}
