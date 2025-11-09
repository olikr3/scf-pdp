use std::f64;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::fmt::{self};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({:.2}, {:.2})", self.x, self.y)
    }
}

// Type-safe indices for different location types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PickupIndex(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DropoffIndex(usize);

#[derive(Debug, PartialEq)]
pub struct Instance {
    name: String,
    n_reqs: usize,
    n_vehicles: usize,
    cap: usize,
    gamma: usize,
    rho: f64,
    demands: Vec<usize>,
    depot: Point,
    pickup_locations: Vec<Point>,
    dropoff_locations: Vec<Point>,
}

impl Instance {
    pub fn from_file(filename: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let path = Path::new(filename);
        
        let instance_name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let first_line = lines.next().ok_or("Empty file")??;
        let first_parts: Vec<&str> = first_line.split_whitespace().collect();
        
        if first_parts.len() < 5 {
            return Err("First line doesn't contain enough parameters".into());
        }
        
        let n_reqs = first_parts[0].parse()?;
        let n_vehicles = first_parts[1].parse()?;
        let cap = first_parts[2].parse()?;
        let gamma = first_parts[3].parse()?;
        let rho = first_parts[4].parse()?;

        let mut current_line = lines.next().ok_or("Missing demands section")??;
        while current_line != "# demands" {
            current_line = lines.next().ok_or("Could not find #demands section")??;
        }

        let demands_line = lines.next().ok_or("No demands data")??;
        let demands: Result<Vec<usize>, _> = demands_line
            .split_whitespace()
            .map(|s| s.parse())
            .collect();
        let demands = demands?;
        
        if demands.len() != n_reqs {
            return Err(format!("Expected {} demands, got {}", n_reqs, demands.len()).into());
        }

        let mut current_line = lines.next().ok_or("Missing locations section")??;
        while current_line != "# request locations" {
            current_line = lines.next().ok_or("Could not find #request locations section")??;
        }

        let depot_line = lines.next().ok_or("Missing depot coordinates")??;
        let depot_parts: Vec<f64> = depot_line
            .split_whitespace()
            .map(|s| s.parse())
            .collect::<Result<_, _>>()?;
        if depot_parts.len() != 2 {
            return Err("Depot coordinates should have 2 values".into());
        }
        let depot = Point { x: depot_parts[0], y: depot_parts[1] };

        let mut pickup_locations = Vec::with_capacity(n_reqs);
        for i in 0..n_reqs {
            let line = lines.next().ok_or(format!("Missing pickup location {}", i))??;
            let parts: Vec<f64> = line
                .split_whitespace()
                .map(|s| s.parse())
                .collect::<Result<_, _>>()?;
            if parts.len() != 2 {
                return Err(format!("Pickup location {} should have 2 values", i).into());
            }
            pickup_locations.push(Point { x: parts[0], y: parts[1] });
        }

        let mut dropoff_locations = Vec::with_capacity(n_reqs);
        for i in 0..n_reqs {
            let line = lines.next().ok_or(format!("Missing drop-off location {}", i))??;
            let parts: Vec<f64> = line
                .split_whitespace()
                .map(|s| s.parse())
                .collect::<Result<_, _>>()?;
            if parts.len() != 2 {
                return Err(format!("Drop-off location {} should have 2 values", i).into());
            }
            dropoff_locations.push(Point { x: parts[0], y: parts[1] });
        }

        Ok(Instance {
            name: instance_name,
            n_reqs,
            n_vehicles,
            cap,
            gamma,
            rho,
            demands,
            depot,
            pickup_locations,
            dropoff_locations,
        })
    }

    pub fn compute_distance_matrix(&self) -> Vec<Vec<usize>> {
        // list of all locations for distance matrix
        let all_locations = self.all_locations();
        let l = all_locations.len();
        let mut dist = vec![vec![0usize; l]; l];

        for u in 0..l {
            for v in 0..l {
                let dx = all_locations[u].x - all_locations[v].x;
                let dy = all_locations[u].y - all_locations[v].y;
                dist[u][v] = dx.hypot(dy).ceil() as usize;
            }
        }
        dist
    }

    /// Get all locations in order: [depot, pickup_0, pickup_1, ..., dropoff_0, dropoff_1, ...]
    pub fn all_locations(&self) -> Vec<Point> {
        let mut all = Vec::with_capacity(1 + 2 * self.n_reqs);
        all.push(self.depot);
        all.extend(&self.pickup_locations);
        all.extend(&self.dropoff_locations);
        all
    }

    pub fn location_description(&self, index: usize) -> String {
        match index {
            0 => "Depot".to_string(),
            i if i <= self.n_reqs => format!("Pickup-{}", i),
            i if i <= 2 * self.n_reqs => format!("Dropoff-{}", i - self.n_reqs),
            _ => "Invalid".to_string(),
        }
    }

    pub fn name(&self) -> &str { &self.name }
    pub fn n_reqs(&self) -> usize { self.n_reqs }
    pub fn n_vehicles(&self) -> usize { self.n_vehicles }
    pub fn cap(&self) -> usize { self.cap }
    pub fn gamma(&self) -> usize { self.gamma }
    pub fn rho(&self) -> f64 { self.rho }
    pub fn demands(&self) -> &Vec<usize> { &self.demands }
    pub fn depot(&self) -> Point { self.depot }
    pub fn pickup_locations(&self) -> &Vec<Point> { &self.pickup_locations }
    pub fn dropoff_locations(&self) -> &Vec<Point> { &self.dropoff_locations }
    
    /// Get pickup location for a specific request
    pub fn pickup_location(&self, request_id: usize) -> Option<Point> {
        if request_id < self.n_reqs {
            Some(self.pickup_locations[request_id])
        } else {
            None
        }
    }
    
    /// Get dropoff location for a specific request
    pub fn dropoff_location(&self, request_id: usize) -> Option<Point> {
        if request_id < self.n_reqs {
            Some(self.dropoff_locations[request_id])
        } else {
            None
        }
    }
}

impl fmt::Display for Instance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Instance: {}", self.name)?;
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
        writeln!(f, "  Depot: {}", self.depot)?;
        
        if self.n_reqs > 0 {
            // Pickup locations
            writeln!(f, "  Pickup locations:")?;
            for i in 0..self.n_reqs {
                writeln!(f, "    Request {}: {}", i + 1, self.pickup_locations[i])?;
            }
            
            // Drop-off locations
            writeln!(f, "  Drop-off locations:")?;
            for i in 0..self.n_reqs {
                writeln!(f, "    Request {}: {}", i + 1, self.dropoff_locations[i])?;
            }
        }
        
        Ok(())
    }
}