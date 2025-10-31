use std::f64;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::error::Error;
use std::path::Path;
use std::fmt::{self, Formatter};


#[derive(Debug, Clone, Copy)]
pub struct Point {
    x: f64,
    y: f64,
}

impl fmt::Display for Point {
    
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({:.2}, {:.2})", self.x, self.y)
    }
}

#[derive(Debug)]
pub struct Instance {
    n_reqs: usize,
    n_vehicles: usize,
    cap: usize,
    gamma: usize,
    rho: usize,
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

    pub fn compute_distance_matrix(&self) -> Vec<Vec<usize>> {
        let l = self.locations.len();
        let mut dist = vec![vec![0usize; l]; l];

        for u in 0..l {
            for v in 0..l {
                let dx = self.locations[u].x - self.locations[v].x;
                let dy = self.locations[u].y - self.locations[v].y;
                dist[u][v] = dx.hypot(dy).ceil() as usize;
            }
        }
        dist
    }

    pub fn n_reqs(&self) -> usize { self.n_reqs }
    pub fn n_vehicles(&self) -> usize { self.n_vehicles }
    pub fn cap(&self) -> usize { self.cap }
    pub fn gamma(&self) -> usize { self.gamma }
    pub fn rho(&self) -> usize { self.rho }
    pub fn demands(&self) -> &Vec<usize> { &self.demands }
    pub fn locations(&self) -> &Vec<Point> { &self.locations }
}


impl fmt::Display for Instance {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Instance Summary:")?;
        writeln!(f, "  Requests: {}", self.n_reqs)?;
        writeln!(f, "  Vehicles: {}", self.n_vehicles)?;
        writeln!(f, "  Capacity: {}", self.cap)?;
        writeln!(f, "  Gamma: {}", self.gamma)?;
        writeln!(f, "  Rho: {}", self.rho)?;
        
        // Demands
        writeln!(f, "  Demands: [{}]", self.demands.iter()
            .map(|d| d.to_string())
            .collect::<Vec<String>>()
            .join(", "))?;
        
        // Locations summary
        writeln!(f, "  Locations ({} total):", self.locations.len())?;
        writeln!(f, "    Depot: {}", self.locations[0])?;
        
        if self.n_reqs > 0 {
            // Pickup locations
            writeln!(f, "    Pickup locations:")?;
            for i in 1..=self.n_reqs {
                writeln!(f, "      Request {}: {}", i, self.locations[i])?;
            }
            
            // Drop-off locations
            writeln!(f, "    Drop-off locations:")?;
            for i in (self.n_reqs + 1)..(2 * self.n_reqs + 1) {
                let req_num = i - self.n_reqs;
                writeln!(f, "      Request {}: {}", req_num, self.locations[i])?;
            }
        }
        
        
        Ok(())

    }
}