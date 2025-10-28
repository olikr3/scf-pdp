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
    
    fn from_file(filename: &str) -> Result<Self> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        
        let first_line = lines.next().unwrap()?
        let first_parts: Vec<&str> = first_line.split_whitespace().collect();
        let n_reqs = first_parts[0].parse()?;
        let n_vehicles = first_parts[1].parse()?;
        let cap = first_parts[2].parse()?;
        let gamma = first_parts[3].parse()?;
        let rho = first_parts[4].parse()?;

        // demands section
        let mut current_line = lines.next().unwrap()?;
        while current_line != "#demands" {
            current_line = lines.next().unwrap()?;
        }
        
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
        for _ in 0..n {
            let line = lines.next().unwrap()?;
            let parts: Vec<f64> = line
                .split_whitespace()
                .map(|s| s.parse().unwrap())
                .collect();
            locations.push(Point { x: parts[0], y: parts[1] });
        }
        
        // Drop-off locations
        for _ in 0..n {
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


}
