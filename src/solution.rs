use std::fmt;
use std::fs::File;
use std::io::{Write};
use std::path::Path;

use crate::instance::Instance;

#[derive(Debug, Clone)]
pub struct Solution<'a> {
    pub instance: &'a Instance,
    pub routes: Vec<Vec<usize>>,
}

impl<'a> Solution<'a> {
    pub fn new(instance: &'a Instance, routes: Vec<Vec<usize>>) -> Self {
        Self {
            instance,
            routes,
        }
    }

    // for beam search
    pub fn empty(instance: &'a Instance, num_vehicles: usize) -> Self {
        Self {
            instance,
            routes: vec![Vec::new(); num_vehicles], // each vehicle has empty route
        }
    }

    pub fn jain_fairness(&self) -> f64 {
        let distances = self.get_route_distances();
        let sum: f64 = distances.iter().sum();
        let sum_sq: f64 = distances.iter().map(|d| d * d).sum();
        let k = distances.len() as f64;

        if sum_sq == 0.0 {
            return 1.0; // all routes empty → perfectly fair
        }

        (sum * sum) / (k * sum_sq)
    }

    pub fn objective_function_value(&self) -> f64 {
        let dist_sum: f64 = self.get_route_distances().iter().sum();
        let fairness = self.jain_fairness();
        dist_sum + self.instance.rho() * (1.0 - fairness)
    }

    pub fn total_travel_distance(&self) -> f64 {
        self.get_route_distances().iter().sum()
    }

    pub fn get_route_distances(&self) -> Vec<f64> {
        let dist_matrix = self.instance.compute_distance_matrix();
        
        self.routes.iter()
            .map(|route| {
                if route.len() < 2 {
                    return 0.0;
                }
                
                let mut distance = 0.0;
                for i in 0..route.len() - 1 {
                    distance += dist_matrix[route[i]][route[i + 1]] as f64;
                }
                distance
            })
            .collect()
    }

    pub fn to_file(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::create(filename)?;
        
        let clean_name = Path::new(self.instance.name())
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(self.instance.name());
        writeln!(file, "{}", clean_name)?;

        // Write each vehicle's route
        for route in &self.routes {
            let route_str: Vec<String> = route.iter().map(|x| x.to_string()).collect();
            writeln!(file, "{}", route_str.join(" "))?;
        }

        Ok(())
    }

    pub fn is_valid(&self) -> bool {
        let n_reqs = self.instance.n_reqs();
        let capacity = self.instance.cap();
        let demands = self.instance.demands();

        // Track which requests are served & by which vehicle
        let mut served_by = vec![None; n_reqs];

        for route in &self.routes {
            let mut load = 0usize;

            for &node in route {
                // Vehicle capacity check: load must be valid at all times
                // If node is a pickup
                if node >= 1 && node <= n_reqs {
                    let req_id = node - 1;
                    load += demands[req_id];
                    if load > capacity {
                        return false;
                    }

                    // Mark request as beginning to be served
                    match served_by[req_id] {
                        None => served_by[req_id] = Some(route.as_ptr()), // track vehicle by pointer
                        Some(_) => return false, // request assigned twice across vehicles
                    }

                // If node is a dropoff
                } else if node > n_reqs && node <= 2 * n_reqs {
                    let req_id = node - 1 - n_reqs;
                    // Must have been picked up before being dropped off in this specific route
                    if served_by[req_id] != Some(route.as_ptr()) {
                        return false;
                    }
                    load -= demands[req_id];
                }
            }
        }

        // Count served requests
        let served_count = served_by.iter().filter(|x| x.is_some()).count();
        served_count >= self.instance.gamma()
    }
}

impl<'a> fmt::Display for Solution<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Solution for instance: {}", self.instance.name())?;
        writeln!(f, "Number of routes: {}", self.routes.len())?;
        writeln!(f, "Total travel distance: {:.2}", self.total_travel_distance())?;
        writeln!(f, "Jain fairness: {:.4}", self.jain_fairness())?;
        writeln!(f, "Objective value: {:.2}", self.objective_function_value())?;
        writeln!(f, "Valid solution: {}", self.is_valid())?;
        
        let route_distances = self.get_route_distances();
        for (i, (route, distance)) in self.routes.iter().zip(route_distances).enumerate() {
            let route_str: Vec<String> = route.iter().map(|x| x.to_string()).collect();
            writeln!(f, "  Vehicle {} (distance: {:.2}): {}", i + 1, distance, route_str.join(" "))?;
        }
        
        Ok(())
    }
}