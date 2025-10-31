use std::collections::HashSet;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Solution {
    pub instance_name: String,
    pub routes: Vec<Vec<usize>>,
}

impl Solution {

    pub fn new(instance_name: String, routes: Vec<Vec<usize>>) -> Self {
        Self {
            instance_name,
            routes,
        }
    }

    pub fn from_file(filename: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let instance_name = lines.next().ok_or("Empty solution file")??;

        let mut routes = Vec::new();
        for line in lines {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            
            let route: Result<Vec<usize>, _> = line
                .split_whitespace()
                .map(|s| s.parse())
                .collect();
            
            routes.push(route?);
        }

        Ok(Solution {
            instance_name,
            routes,
        })
    }


    pub fn to_file(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::create(filename)?;
        
        let clean_name = Path::new(&self.instance_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&self.instance_name);
        writeln!(file, "{}", clean_name)?;

        // Write each vehicle's route
        for route in &self.routes {
            let route_str: Vec<String> = route.iter().map(|x| x.to_string()).collect();
            writeln!(file, "{}", route_str.join(" "))?;
        }

        Ok(())
    }
}

impl fmt::Display for Solution {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Solution for instance: {}", self.instance_name)?;
        writeln!(f, "Number of routes: {}", self.routes.len())?;
        
        for (i, route) in self.routes.iter().enumerate() {
            let route_str: Vec<String> = route.iter().map(|x| x.to_string()).collect();
            writeln!(f, "  Vehicle {}: {}", i + 1, route_str.join(" "))?;
        }
        
        Ok(())
    }
}