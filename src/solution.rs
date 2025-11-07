use std::collections::HashSet;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
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
        let route_distances = self.get_route_distances();
        let n = route_distances.len();
        
        if n == 0 {
            return 1.0;
        }
        
        let sum_distances: f64 = route_distances.iter().sum();
        let sum_squared_distances: f64 = route_distances.iter().map(|&d| d * d).sum();
        
        if sum_squared_distances == 0.0 {
            return 1.0;
        }
        
        (sum_distances * sum_distances) / (n as f64 * sum_squared_distances)
    }

    pub fn objective_function_value(&self) -> f64 {
        let total_distance: f64 = self.total_travel_distance();
        let jain_fairness = self.jain_fairness();
        
        total_distance + self.instance.rho() * (1.0 - jain_fairness)
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

    // to check if solution is valid
    pub fn is_valid(&self) -> bool {
        todo!()
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