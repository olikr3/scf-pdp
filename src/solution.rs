use std::fmt;
use std::fs::File;
use std::io::{Write};
use std::path::Path;

use crate::instance::Instance;

#[derive(Debug, Clone)]
pub struct Solution {
    pub instance: Instance,  // Owned instead of borrowed
    pub routes: Vec<Vec<usize>>,
}

impl Solution {
    pub fn new(instance: Instance, routes: Vec<Vec<usize>>) -> Self {
        Self {
            instance,
            routes,
        }
    }

    // for beam search
    pub fn empty(instance: Instance, num_vehicles: usize) -> Self {
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

        // numeric safety
        let eps = 1e-12;
        if sum_sq <= eps {
            return 1.0; // all routes empty (or numerically zero) → perfectly fair
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
                if route.is_empty() {
                    return 0.0;
                }
                
                let mut distance = 0.0;
                
                // Distance from depot to first stop
                if route[0] != 0 { // if first stop is not depot
                    distance += dist_matrix[0][route[0]] as f64;
                }
                
                // Distance between consecutive stops
                for i in 0..route.len() - 1 {
                    distance += dist_matrix[route[i]][route[i + 1]] as f64;
                }
                
                // Distance from last stop back to depot
                if let Some(&last_stop) = route.last() {
                    if last_stop != 0 { // if last stop is not depot
                        distance += dist_matrix[last_stop][0] as f64;
                    }
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

        // Write each vehicle's route (only request location indices, no depot)
        for route in &self.routes {
            if route.is_empty() {
                writeln!(file)?;
                continue;
            }
            
            // Filter out depot (index 0) and write only request locations
            let request_stops: Vec<String> = route
                .iter()
                .filter(|&&stop| stop != 0)
                .map(|stop| stop.to_string())
                .collect();
                
            writeln!(file, "{}", request_stops.join(" "))?;
        }

        Ok(())
    }

    pub fn is_valid(&self) -> bool {
        let n_reqs = self.instance.n_reqs();
        let capacity = self.instance.cap();
        let demands = self.instance.demands();
        let gamma = self.instance.gamma();

        // Track which requests are served & by which vehicle
        let mut served_by = vec![None; n_reqs];
        let mut served_count = 0;

        for (vehicle_id, route) in self.routes.iter().enumerate() {
            let mut load = 0usize;
            let mut picked_up = vec![false; n_reqs]; // Track pickups within this route

            for &node in route {
                // Skip depot nodes for request tracking (but still important for capacity)
                if node == 0 {
                    continue;
                }
                
                // If node is a pickup (indices 1 to n_reqs)
                if node >= 1 && node <= n_reqs {
                    let req_id = node - 1;
                    
                    // Check if request already served by another vehicle
                    if served_by[req_id].is_some() {
                        return false;
                    }
                    
                    // Check capacity
                    if load + demands[req_id] > capacity {
                        return false;
                    }
                    
                    load += demands[req_id];
                    picked_up[req_id] = true;
                    served_by[req_id] = Some(vehicle_id);

                // If node is a dropoff (indices n_reqs+1 to 2*n_reqs)
                } else if node > n_reqs && node <= 2 * n_reqs {
                    let req_id = node - n_reqs - 1;
                    
                    // Check if pickup happened in this route
                    if !picked_up[req_id] {
                        return false;
                    }
                    
                    // Check if dropoff happens after pickup in the same route
                    if served_by[req_id] != Some(vehicle_id) {
                        return false;
                    }
                    
                    load -= demands[req_id];
                    served_count += 1;
                }
            }
            
            // Final load check - should be zero at the end of route
            if load != 0 {
                return false;
            }
        }

        // Check if we served at least gamma requests
        served_count >= gamma
    }
}

impl fmt::Display for Solution {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Solution for instance: {}", self.instance.name())?;
        writeln!(f, "Number of routes: {}", self.routes.len())?;
        writeln!(f, "Total travel distance: {:.2}", self.total_travel_distance())?;
        writeln!(f, "Jain fairness: {:.4}", self.jain_fairness())?;
        writeln!(f, "Objective value: {:.2}", self.objective_function_value())?;
        writeln!(f, "Valid solution: {}", self.is_valid())?;
        
        let route_distances = self.get_route_distances();
        for (i, (route, distance)) in self.routes.iter().zip(route_distances).enumerate() {
            let route_with_depot: Vec<String> = route.iter().map(|x| {
                if *x == 0 {
                    "depot".to_string()
                } else {
                    self.instance.location_description(*x)
                }
            }).collect();
            writeln!(f, "  Vehicle {} (distance: {:.2}): {}", 
                     i + 1, distance, route_with_depot.join(" → "))?;
        }
        
        Ok(())
    }
}