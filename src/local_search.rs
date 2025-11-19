use std::time::Instant;
use crate::{DeterministicConstruction, Instance, Solution, Solver};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Neighborhood {
    Relocate,    // Move a request from one route to another
    Exchange,    // Swap two requests between routes
    TwoOpt,      // Reverse a segment within a route
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StepFunction {
    FirstImprovement,
    BestImprovement,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AcceptanceCriterion {
    ImprovingOnly,
}

#[derive(Debug, Clone)]
pub struct LocalSearchConfig {
    pub neighborhood: Neighborhood,
    pub step_function: StepFunction,
    pub acceptance: AcceptanceCriterion,
    pub max_iterations: usize,
    pub max_no_improvement: usize,
    pub time_limit_seconds: u64,
}

impl Default for LocalSearchConfig {
    fn default() -> Self {
        Self {
            neighborhood: Neighborhood::Exchange,
            step_function: StepFunction::FirstImprovement,
            acceptance: AcceptanceCriterion::ImprovingOnly,
            max_iterations: 1000,
            max_no_improvement: 100,
            time_limit_seconds: 60,
        }
    }
}

pub struct LocalSearch<'a> {
    instance: &'a Instance,
    config: LocalSearchConfig,
}

impl<'a> LocalSearch<'a> {
    pub fn new(instance: &'a Instance, config: LocalSearchConfig) -> Self {
        Self { instance, config }
    }

    fn construct_initial_solution(&self) -> Solution {
        let det = DeterministicConstruction::new(self.instance);
        det.solve()
    }

    fn relocate_nh(&self, current: &Solution) -> Vec<Solution> {
        let mut neighbors = Vec::new();
        let n_vehicles = current.routes.len();
        
        for v1 in 0..n_vehicles {
            for v2 in 0..n_vehicles {
                if v1 == v2 {
                    continue;
                }
                
                // Try to relocate each request from v1 to v2
                let requests_in_v1 = self.extract_requests_from_route(&current.routes[v1]);
                
                for req in requests_in_v1 {
                    // Remove request from v1
                    let route_v1_without_req = self.remove_request_from_route(&current.routes[v1], req);
                    
                    // Try to insert request into v2 at all possible positions
                    for insert_pos in 0..=current.routes[v2].len() {
                        if let Some(route_v2_with_req) = self.insert_request_into_route(
                            &current.routes[v2], 
                            req, 
                            insert_pos
                        ) {
                            // Create new solution with modified routes
                            let mut new_routes = current.routes.clone();
                            new_routes[v1] = route_v1_without_req.clone();
                            new_routes[v2] = route_v2_with_req;
                            
                            let new_solution = Solution::new(current.instance.clone(), new_routes);
                            
                            if new_solution.is_valid() {
                                neighbors.push(new_solution);
                            }
                        }
                    }
                }
            }
        }
        
        neighbors
    }

    fn exchange_nh(&self, current: &Solution) -> Vec<Solution> {
        let mut neighbors = Vec::new();
        let n_vehicles = current.routes.len();
        
        for v1 in 0..n_vehicles {
            for v2 in v1 + 1..n_vehicles {
                let requests_v1 = self.extract_requests_from_route(&current.routes[v1]);
                let requests_v2 = self.extract_requests_from_route(&current.routes[v2]);
                
                for &req1 in &requests_v1 {
                    for &req2 in &requests_v2 {
                        // Remove req1 from v1 and req2 from v2
                        let route_v1_without_req1 = self.remove_request_from_route(&current.routes[v1], req1);
                        let route_v2_without_req2 = self.remove_request_from_route(&current.routes[v2], req2);
                        
                        // Try all combinations of inserting req1 into v2 and req2 into v1
                        for pos1 in 0..=route_v2_without_req2.len() {
                            for pos2 in 0..=route_v1_without_req1.len() {
                                if let (Some(route_v1_with_req2), Some(route_v2_with_req1)) = (
                                    self.insert_request_into_route(&route_v1_without_req1, req2, pos2),
                                    self.insert_request_into_route(&route_v2_without_req2, req1, pos1)
                                ) {
                                    // Create new solution with modified routes
                                    let mut new_routes = current.routes.clone();
                                    new_routes[v1] = route_v1_with_req2;
                                    new_routes[v2] = route_v2_with_req1;
                                    
                                    let new_solution = Solution::new(current.instance.clone(), new_routes);
                                    
                                    if new_solution.is_valid() {
                                        neighbors.push(new_solution);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        neighbors
    }

    fn two_opt_nh(&self, current: &Solution) -> Vec<Solution> {
        let mut neighbors = Vec::new();
        
        for v in 0..current.routes.len() {
            let route = &current.routes[v];
            if route.len() < 4 {
                continue; // Need at least 4 stops for 2-opt to make sense
            }
            
            for i in 1..route.len() - 2 {
                for j in i + 1..route.len() - 1 {
                    let mut new_route = route[0..i].to_vec();
                    
                    // Reverse the segment between i and j
                    let reversed_segment: Vec<usize> = route[i..=j].iter().rev().cloned().collect();
                    new_route.extend(reversed_segment);
                    
                    new_route.extend(&route[j + 1..]);
                    
                    // Create new solution with modified route
                    let mut new_routes = current.routes.clone();
                    new_routes[v] = new_route;
                    
                    let new_solution = Solution::new(current.instance.clone(), new_routes);
                    
                    if new_solution.is_valid() {
                        neighbors.push(new_solution);
                    }
                }
            }
        }
        
        neighbors
    }

    fn extract_requests_from_route(&self, route: &[usize]) -> Vec<usize> {
        let mut requests = Vec::new();
        let n_reqs = self.instance.n_reqs();
        
        for &node in route {
            if node != 0 { // Skip depot
                if node <= n_reqs {
                    // This is a pickup node, the request ID is node - 1
                    requests.push(node - 1);
                }
            }
        }
        
        // Remove duplicates (since each request appears twice: pickup and dropoff)
        requests.sort();
        requests.dedup();
        requests
    }

    fn remove_request_from_route(&self, route: &[usize], request_id: usize) -> Vec<usize> {
        let pickup_node = request_id + 1;
        let dropoff_node = request_id + 1 + self.instance.n_reqs();
        
        route.iter()
            .filter(|&&node| node != pickup_node && node != dropoff_node)
            .cloned()
            .collect()
    }

    fn insert_request_into_route(&self, route: &[usize], request_id: usize, position: usize) -> Option<Vec<usize>> {
        let pickup_node = request_id + 1;
        let dropoff_node = request_id + 1 + self.instance.n_reqs();
        let _demand = self.instance.demands()[request_id];
        let capacity = self.instance.cap();
        
        // Check if we can insert both pickup and dropoff while maintaining capacity
        let mut test_route = route.to_vec();
        
        // Insert pickup at position
        test_route.insert(position, pickup_node);
        
        // Find valid positions for dropoff after pickup
        for dropoff_pos in position + 1..=test_route.len() {
            let mut final_route = test_route.clone();
            final_route.insert(dropoff_pos, dropoff_node);
            
            // Check capacity constraints
            if self.check_route_capacity(&final_route, capacity) {
                return Some(final_route);
            }
        }
        
        None
    }

    fn check_route_capacity(&self, route: &[usize], capacity: usize) -> bool {
        let mut current_load = 0;
        let n_reqs = self.instance.n_reqs();
        let demands = self.instance.demands();
        
        for &node in route {
            if node == 0 {
                continue; // Depot doesn't affect load
            }
            
            if node <= n_reqs {
                // Pickup node
                let req_id = node - 1;
                current_load += demands[req_id];
            } else if node <= 2 * n_reqs {
                // Dropoff node
                let req_id = node - n_reqs - 1;
                current_load -= demands[req_id];
            }
            
            if current_load > capacity {
                return false;
            }
        }
        
        true
    }

    fn generate_neighbors(&self, current: &Solution) -> Vec<Solution> {
        match self.config.neighborhood {
            Neighborhood::Relocate => self.relocate_nh(current),
            Neighborhood::Exchange => self.exchange_nh(current),
            Neighborhood::TwoOpt => self.two_opt_nh(current),
        }
    }

    fn search_step(&self, current: &Solution) -> Option<Solution> {
        let neighbors = self.generate_neighbors(current);
        let current_obj = current.objective_function_value();
        
        match self.config.step_function {
            StepFunction::FirstImprovement => {
                for neighbor in neighbors {
                    if neighbor.objective_function_value() < current_obj {
                        return Some(neighbor);
                    }
                }
                None
            }
            StepFunction::BestImprovement => {
                let mut best_neighbor = None;
                let mut best_obj = current_obj;
                
                for neighbor in neighbors {
                    let neighbor_obj = neighbor.objective_function_value();
                    if neighbor_obj < best_obj {
                        best_obj = neighbor_obj;
                        best_neighbor = Some(neighbor);
                    }
                }
                
                best_neighbor
            }
        }
    }
}

impl<'a> Solver for LocalSearch<'a> {
    fn solve(&self) -> Solution {
        let start_time = Instant::now();
        let mut current = self.construct_initial_solution();
        let mut best_solution = current.clone();
        let mut best_obj = current.objective_function_value();
        
        let mut iterations = 0;
        let mut no_improvement_count = 0;
        
        while iterations < self.config.max_iterations 
            && no_improvement_count < self.config.max_no_improvement
            && start_time.elapsed().as_secs() < self.config.time_limit_seconds {
            
            if let Some(neighbor) = self.search_step(&current) {
                current = neighbor;
                
                let current_obj = current.objective_function_value();
                if current_obj < best_obj {
                    best_solution = current.clone();
                    best_obj = current_obj;
                    no_improvement_count = 0;
                } else {
                    no_improvement_count += 1;
                }
            } else {
                // No improving neighbor found
                break;
            }
            
            iterations += 1;
        }
        
        best_solution
    }
}