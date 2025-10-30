use std::f64;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub struct Point {
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
    
    pub fn from_file(filename: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        
        // Parse first line
        let first_line = lines.next().unwrap()?;
        let first_parts: Vec<&str> = first_line.split_whitespace().collect();
        let n_reqs = first_parts[0].parse()?;
        let n_vehicles = first_parts[1].parse()?;
        let cap = first_parts[2].parse()?;
        let gamma = first_parts[3].parse()?;
        let rho = first_parts[4].parse()?;
        
        // Skip to demands section
        let mut current_line = lines.next().unwrap()?;
        while current_line != "#demands" {
            current_line = lines.next().unwrap()?;
        }
        
        // Parse demands
        let demands_line = lines.next().unwrap()?;
        let demands: Vec<usize> = demands_line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        
        // Skip to locations section
        current_line = lines.next().unwrap()?;
        while current_line != "#request locations" {
            current_line = lines.next().unwrap()?;
        }
        
        // Parse locations
        let mut locations = Vec::new();
        
        // Depot
        let depot_line = lines.next().unwrap()?;
        let depot_parts: Vec<f64> = depot_line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        locations.push(Point { x: depot_parts[0], y: depot_parts[1] });
        
        // Pickup locations
        for _ in 0..n_reqs {
            let line = lines.next().unwrap()?;
            let parts: Vec<f64> = line
                .split_whitespace()
                .map(|s| s.parse().unwrap())
                .collect();
            locations.push(Point { x: parts[0], y: parts[1] });
        }
        
        // Drop-off locations
        for _ in 0..n_reqs {
            let line = lines.next().unwrap()?;
            let parts: Vec<f64> = line
                .split_whitespace()
                .map(|s| s.parse().unwrap())
                .collect();
            locations.push(Point { x: parts[0], y: parts[1] });
        }
        
        Ok(Instance {
            n_reqs,
            n_vehicles,
            cap,
            gamma,
            rho,
            demands,
            locations,
        })
    }


    pub fn n_reqs(&self) -> usize { self.n_reqs }
    pub fn n_vehicles(&self) -> usize { self.n_vehicles }
    pub fn cap(&self) -> usize { self.cap }
    pub fn gamma(&self) -> usize { self.gamma }
    pub fn rho(&self) -> usize { self.rho }
    pub fn demands(&self) -> &Vec<usize> { &self.demands }
    pub fn locations(&self) -> &Vec<Point> { &self.locations }

}
