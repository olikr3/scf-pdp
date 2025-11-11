use rand::prelude::*;
use crate::{Instance, Solution, Solver};

/*
Samples gamma requests at random and assigns them to vehicles uniformally random if they have spare capacity.
Can be biased towards routes closer to the depot.
*/
pub struct RandomConstruction<'a> {
    instance: &'a Instance,
    biased: bool,
}


impl<'a> RandomConstruction<'a> {
    
    pub fn new(instance: &'a Instance, biased: bool) -> Self {
        Self { instance, biased }
    }

    fn construct_solution(&self) -> Solution<'a> {
        let mut rng = thread_rng();
        let n_reqs = self.instance.n_reqs();
        let n_vehicles = self.instance.n_vehicles();
        let capacity = self.instance.cap();
        let gamma = self.instance.gamma();
        let demands = self.instance.demands().clone();
        
        let mut solution = Solution::empty(self.instance, n_vehicles);
        
        let mut all_requests: Vec<usize> = (0..n_reqs).collect();
        all_requests.shuffle(&mut rng);
        let selected_requests: Vec<usize> = all_requests.into_iter().take(gamma).collect();
        
        if self.biased {
            self.assign_requests_biased(&mut solution, &selected_requests, &demands, capacity, &mut rng);
        } else {
            self.assign_requests_uniform(&mut solution, &selected_requests, &demands, capacity, &mut rng);
        }
        
        solution
    }

    fn assign_requests_uniform(
        &self,
        solution: &mut Solution<'a>,
        selected_requests: &[usize],
        demands: &[usize],
        capacity: usize,
        rng: &mut ThreadRng,
    ) {
        let n_vehicles = solution.routes.len();
        
        for &req_id in selected_requests {
            let mut assigned = false;
            let mut vehicle_order: Vec<usize> = (0..n_vehicles).collect();
            vehicle_order.shuffle(rng);
            
            // Try random assignment
            for &vehicle_id in &vehicle_order {
                if self.can_assign_request(vehicle_id, req_id, solution, demands, capacity) {
                    self.assign_request_to_vehicle(vehicle_id, req_id, solution);
                    assigned = true;
                    break;
                }
            }
            
            // If no vehicle can take it due to capacity, assign to a random vehicle
            // (this might create invalid solutions, but  validation is handled later)
            if !assigned {
                let vehicle_id = vehicle_order[0];
                self.assign_request_to_vehicle(vehicle_id, req_id, solution);
            }
        }
    }

    fn assign_requests_biased(
        &self,
        solution: &mut Solution<'a>,
        selected_requests: &[usize],
        demands: &[usize],
        capacity: usize,
        rng: &mut ThreadRng,
    ) {
        let n_vehicles = solution.routes.len();
        let n_reqs = self.instance.n_reqs();
        let dist_matrix = self.instance.compute_distance_matrix();
        
        // Calculate depot proximity scores for each vehicle based on current routes
        let mut vehicle_scores = vec![0.0; n_vehicles];
        
        for vehicle_id in 0..n_vehicles {
            let route = &solution.routes[vehicle_id];
            if route.is_empty() {
                // Empty route gets high score (prefer to use empty vehicles)
                vehicle_scores[vehicle_id] = 1.0;
            } else {
                // Calculate average distance from depot to current stops
                let mut total_dist = 0.0;
                for &stop in route {
                    total_dist += dist_matrix[0][stop] as f64;
                }
                let avg_dist = total_dist / route.len() as f64;
                // Lower distance = higher score (closer to depot)
                vehicle_scores[vehicle_id] = 1.0 / (1.0 + avg_dist);
            }
        }
        
        for &req_id in selected_requests {
            // Create weighted distribution based on scores
            let total_score: f64 = vehicle_scores.iter().sum();
            let mut vehicle_probs: Vec<(usize, f64)> = Vec::new();
            
            if total_score > 0.0 {
                let mut cum_prob = 0.0;
                for (vehicle_id, &score) in vehicle_scores.iter().enumerate() {
                    let prob = score / total_score;
                    cum_prob += prob;
                    vehicle_probs.push((vehicle_id, cum_prob));
                }
            } else {
                // If all scores are zero, use uniform distribution
                for vehicle_id in 0..n_vehicles {
                    vehicle_probs.push((vehicle_id, (vehicle_id + 1) as f64 / n_vehicles as f64));
                }
            }
            
            vehicle_probs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            
            let mut assigned = false;
            for &(vehicle_id, _) in &vehicle_probs {
                if self.can_assign_request(vehicle_id, req_id, solution, demands, capacity) {
                    self.assign_request_to_vehicle(vehicle_id, req_id, solution);
                    
                    // Update score for this vehicle (penalize for adding more work)
                    vehicle_scores[vehicle_id] *= 0.8; // Reduce score to avoid overloading
                    
                    assigned = true;
                    break;
                }
            }
            
            // If no suitable vehicle found, assign to highest probability vehicle
            if !assigned && !vehicle_probs.is_empty() {
                let vehicle_id = vehicle_probs[0].0;
                self.assign_request_to_vehicle(vehicle_id, req_id, solution);
                vehicle_scores[vehicle_id] *= 0.8;
            }
        }
    }

    fn can_assign_request(
        &self,
        vehicle_id: usize,
        req_id: usize,
        solution: &Solution<'a>,
        demands: &[usize],
        capacity: usize,
    ) -> bool {
        let route = &solution.routes[vehicle_id];
        let demand = demands[req_id];
        
        // Calculate current load in the route
        let mut current_load = 0;
        for &stop in route {
            if stop == 0 {
                continue; // depot
            }
            
            // If it's a pickup, add demand
            if stop >= 1 && stop <= self.instance.n_reqs() {
                let stop_req_id = stop - 1;
                current_load += demands[stop_req_id];
            }
            // If it's a dropoff, subtract demand
            else if stop > self.instance.n_reqs() && stop <= 2 * self.instance.n_reqs() {
                let stop_req_id = stop - self.instance.n_reqs() - 1;
                current_load -= demands[stop_req_id];
            }
        }
        current_load + demand <= capacity
    }

    fn assign_request_to_vehicle(
        &self,
        vehicle_id: usize,
        req_id: usize,
        solution: &mut Solution<'a>,
    ) {
        let pickup_loc = req_id + 1;
        let dropoff_loc = req_id + 1 + self.instance.n_reqs();
        
        let route = &mut solution.routes[vehicle_id];
        
        // Simple strategy: add pickup then dropoff at the end
        // In a more sophisticated version, we could insert them in better positions
        route.push(pickup_loc);
        route.push(dropoff_loc);
    }
}

impl<'a> Solver for RandomConstruction<'a> {
    fn solve(&self) -> Solution {
        self.construct_solution()
    }
}