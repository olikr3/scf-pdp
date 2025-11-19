use crate::{Instance, Solution, Solver};
use std::collections::HashMap;

pub struct DeterministicConstruction<'a> {
    instance: &'a Instance,
}

impl<'a> DeterministicConstruction<'a> {
    pub fn new(instance: &'a Instance) -> Self {
        Self { instance }
    }

    /* 
    Simple construction heuristic: serve first gamma requests
    */
    fn construct_solution(&self) -> Solution {
        let n_reqs = self.instance.n_reqs();
        let gamma = self.instance.gamma();
        let n_vehicles = self.instance.n_vehicles();
        let capacity = self.instance.cap();
        let demands = self.instance.demands();
        let dist_matrix = self.instance.compute_distance_matrix();

        // Initialize empty routes (will add depot at start/end later)
        let mut routes: Vec<Vec<usize>> = vec![Vec::new(); n_vehicles];
        let mut loads = vec![0usize; n_vehicles];

        // Helper functions for node indices
        // In the distance matrix: 
        // - index 0 = depot
        // - indices 1..=n_reqs = pickup locations  
        // - indices n_reqs+1..=2*n_reqs = dropoff locations
        let pickup_index = |req_id: usize| -> usize { 1 + req_id };
        let dropoff_index = |req_id: usize| -> usize { 1 + n_reqs + req_id };

        // Helper to compute complete route distance including depot start/end
        let compute_route_distance = |route: &[usize]| -> f64 {
            if route.is_empty() {
                return 0.0;
            }
            
            let mut distance = 0.0;
            
            // From depot to first stop
            distance += dist_matrix[0][route[0]] as f64;
            
            // Between stops
            for i in 0..route.len() - 1 {
                distance += dist_matrix[route[i]][route[i + 1]] as f64;
            }
            
            // From last stop back to depot
            distance += dist_matrix[route[route.len() - 1]][0] as f64;
            
            distance
        };

        // Compute Jain fairness for current routes
        let compute_fairness = |routes: &[Vec<usize>]| -> f64 {
            let distances: Vec<f64> = routes.iter()
                .map(|route| compute_route_distance(route))
                .collect();
                
            let sum: f64 = distances.iter().sum();
            let sum_sq: f64 = distances.iter().map(|d| d * d).sum();
            let k = distances.len() as f64;

            if sum_sq == 0.0 {
                1.0
            } else {
                (sum * sum) / (k * sum_sq)
            }
        };

        // Try to assign each of the first gamma requests
        for req_id in 0..gamma.min(n_reqs) {
            let demand = demands[req_id];
            let pickup = pickup_index(req_id);
            let dropoff = dropoff_index(req_id);

            let mut best_vehicle = None;
            let mut best_score = f64::INFINITY;

            // Try each vehicle
            for k in 0..n_vehicles {
                // Check capacity constraint
                if loads[k] + demand > capacity {
                    continue;
                }

                // Create test route by adding pickup and dropoff
                let mut test_route = routes[k].clone();
                test_route.push(pickup);
                test_route.push(dropoff);

                // Compute new distances for all vehicles
                let mut test_routes = routes.clone();
                test_routes[k] = test_route;

                let current_fairness = compute_fairness(&routes);
                let new_fairness = compute_fairness(&test_routes);
                
                let current_distance: f64 = routes.iter()
                    .map(|r| compute_route_distance(r))
                    .sum();
                let new_distance: f64 = test_routes.iter()
                    .map(|r| compute_route_distance(r))
                    .sum();
                
                let delta_distance = new_distance - current_distance;
                let delta_fairness = new_fairness - current_fairness;
                
                // Objective: minimize total_distance + rho * (1 - fairness)
                // So we want to minimize: delta_distance - rho * delta_fairness
                let score = delta_distance - self.instance.rho() * delta_fairness;

                if score < best_score {
                    best_score = score;
                    best_vehicle = Some(k);
                }
            }

            // Assign to best vehicle found, or to first available vehicle as fallback
            let vehicle = best_vehicle.unwrap_or_else(|| {
                // Fallback: choose vehicle with smallest current load
                loads.iter()
                    .enumerate()
                    .min_by_key(|(_, &load)| load)
                    .map(|(idx, _)| idx)
                    .unwrap_or(0)
            });

            // Add pickup and dropoff to the chosen vehicle's route
            routes[vehicle].push(pickup_index(req_id));
            routes[vehicle].push(dropoff_index(req_id));
            loads[vehicle] += demand;
        }

        // Note: We don't explicitly add depot to routes since the distance calculation
        // already accounts for depot start/end. The solution format expects only request locations.
        
        Solution::new(self.instance.clone(), routes)
    }
}

impl<'a> Solver for DeterministicConstruction<'a> {
    fn solve(&self) -> Solution {
        self.construct_solution()
    }
}

// utility based heuristic
impl<'a> DeterministicConstruction<'a> {

    /*
    uses compute_utility() to rank routes based on a bias towards closeness to depot.
    then it distributes them uniformly among vehicles.
     */
    pub fn utility_based_construction(&self) -> Solution {
        let n_reqs = self.instance.n_reqs();
        let gamma = self.instance.gamma().min(n_reqs);
        let n_vehicles = self.instance.n_vehicles();
        let capacity = self.instance.cap();
        let demands = self.instance.demands();
        
        let utility_map = self.compute_utility();
        
        // Create a sorted list of requests by utility (highest first)
        let mut requests_with_utility: Vec<(usize, f64)> = utility_map.into_iter().collect();
        requests_with_utility.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Take top gamma requests
        let top_gamma_reqs: Vec<usize> = requests_with_utility
            .into_iter()
            .take(gamma)
            .map(|(req_id, _)| req_id)
            .collect();
        
        // Initialize routes and loads
        let mut routes: Vec<Vec<usize>> = vec![Vec::new(); n_vehicles];
        let mut loads = vec![0usize; n_vehicles];
        
        // Helper functions for node indices
        let pickup_index = |req_id: usize| -> usize { 1 + req_id };
        let dropoff_index = |req_id: usize| -> usize { 1 + n_reqs + req_id };
        
        // Distribute requests evenly among vehicles
        for (i, &req_id) in top_gamma_reqs.iter().enumerate() {
            let vehicle = i % n_vehicles;
            let demand = demands[req_id];
            
            // Check capacity constraint
            if loads[vehicle] + demand <= capacity {
                routes[vehicle].push(pickup_index(req_id));
                routes[vehicle].push(dropoff_index(req_id));
                loads[vehicle] += demand;
            } else {
                // If capacity exceeded, try to find another vehicle with capacity
                let mut assigned = false;
                for v in 0..n_vehicles {
                    if loads[v] + demand <= capacity {
                        routes[v].push(pickup_index(req_id));
                        routes[v].push(dropoff_index(req_id));
                        loads[v] += demand;
                        assigned = true;
                        break;
                    }
                }
                
                // If no vehicle has capacity, skip this request
                if !assigned {
                    continue;
                }
            }
        }
        
        Solution::new(self.instance.clone(), routes)
    }

    /*
    Associates each request with a utility score, defined as follows:
    u_i = (c_i)/ (distance depot-pickup + pickup-dropoff-distance + distance drop_off-depot
    That way we bias in favor of routes closer to the depot
     */
    fn compute_utility(&self) -> HashMap<usize, f64> {
        let demands = self.instance.demands();
        let dist_matrix = self.instance.compute_distance_matrix();
        let n_reqs = self.instance.n_reqs();
        
        let mut utility_scores = HashMap::new();
        
        for request_id in 0..n_reqs {
            let pickup_idx = request_id + 1;
            let dropoff_idx = request_id + 1 + n_reqs;
            
            let depot_to_pickup = dist_matrix[0][pickup_idx] as f64;
            let pickup_to_dropoff = dist_matrix[pickup_idx][dropoff_idx] as f64;
            let dropoff_to_depot = dist_matrix[dropoff_idx][0] as f64;
            
            let total_distance = depot_to_pickup + pickup_to_dropoff + dropoff_to_depot;
            
            // Avoid division by zero
            let utility = if total_distance > 0.0 {
                demands[request_id] as f64 / total_distance
            } else {
                demands[request_id] as f64
            };
            
            utility_scores.insert(request_id, utility);
        }
        
        utility_scores
    }
}