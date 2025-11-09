// beam_search.rs
use crate::{Instance, Solution, Solver};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReqState {
    Unserved,
    PickedUp,
    Delivered,
}

#[derive(Debug, Clone, PartialEq)]
struct PartialSolution<'a> {
    instance: &'a Instance,
    routes: Vec<Vec<usize>>,
    req_states: Vec<ReqState>,
    served_count: usize,
    current_loads: Vec<usize>,
    // Track which vehicle picked up each request
    pickup_vehicle: Vec<Option<usize>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BeamSearch<'a> {
    instance: &'a Instance,
    beam_width: usize,
    max_depth: Option<usize>,
}

impl<'a> BeamSearch<'a> {
    pub fn new(instance: &'a Instance) -> Self {
        Self {
            instance,
            beam_width: 20, // Increased for better exploration
            max_depth: Some(200),
        }
    }

    pub fn with_beam_width(mut self, beam_width: usize) -> Self {
        self.beam_width = beam_width;
        self
    }

    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = Some(max_depth);
        self
    }

    pub fn search(&self) -> Solution<'a> {
        let mut beam = vec![self.initial_state()];
        let dist_matrix = self.instance.compute_distance_matrix();
        let max_depth = self.max_depth.unwrap_or(self.instance.n_reqs() * 4);

        for depth in 0..max_depth {
            if beam.is_empty() { 
                break; 
            }

            let mut all_successors = Vec::new();
            for state in &beam {
                let successors = self.generate_successors(state, &dist_matrix);
                all_successors.extend(successors);
            }

            if all_successors.is_empty() { 
                break; 
            }

            // Keep complete solutions that meet gamma requirement
            let (complete, incomplete): (Vec<_>, Vec<_>) = all_successors
                .into_iter()
                .partition(|s| s.served_count >= self.instance.gamma() && self.is_feasible(s));

            if !complete.is_empty() {
                beam = self.select_best_states(complete, &dist_matrix);
                // Continue searching for better complete solutions
                if depth < max_depth - 1 {
                    let mut continued_beam = beam.clone();
                    for state in &beam {
                        let more_successors = self.generate_successors(state, &dist_matrix);
                        continued_beam.extend(more_successors);
                    }
                    beam = self.select_best_states(continued_beam, &dist_matrix);
                }
                break;
            } else {
                beam = self.select_best_states(incomplete, &dist_matrix);
            }
        }

        self.best_complete_solution(&beam, &dist_matrix)
            .unwrap_or_else(|| self.fallback_solution())
    }

    fn initial_state(&self) -> PartialSolution<'a> {
        PartialSolution {
            instance: self.instance,
            routes: vec![Vec::new(); self.instance.n_vehicles()],
            req_states: vec![ReqState::Unserved; self.instance.n_reqs()],
            served_count: 0,
            current_loads: vec![0; self.instance.n_vehicles()],
            pickup_vehicle: vec![None; self.instance.n_reqs()],
        }
    }

    fn generate_successors(&self, state: &PartialSolution<'a>, dist_matrix: &[Vec<usize>]) -> Vec<PartialSolution<'a>> {
        let mut successors = Vec::new();

        for vehicle_id in 0..self.instance.n_vehicles() {
            let route = &state.routes[vehicle_id];
            let current_load = state.current_loads[vehicle_id];
            let last_loc = route.last().copied().unwrap_or(0);

            // 1. Pickup actions
            for req_id in 0..self.instance.n_reqs() {
                if state.req_states[req_id] == ReqState::Unserved {
                    let demand = self.instance.demands()[req_id];
                    if current_load + demand <= self.instance.cap() {
                        if let Some(new_state) = self.apply_pickup(state, vehicle_id, req_id) {
                            successors.push(new_state);
                        }
                    }
                }
            }

            // 2. Dropoff actions (only for requests picked up by this vehicle)
            for req_id in 0..self.instance.n_reqs() {
                if state.req_states[req_id] == ReqState::PickedUp {
                    if state.pickup_vehicle[req_id] == Some(vehicle_id) {
                        if let Some(new_state) = self.apply_dropoff(state, vehicle_id, req_id) {
                            successors.push(new_state);
                        }
                    }
                }
            }

            // 3. Return to depot ONLY when necessary (vehicle full or no more feasible actions)
            if !route.is_empty() && last_loc != 0 {
                let has_feasible_pickups = (0..self.instance.n_reqs()).any(|req_id| {
                    state.req_states[req_id] == ReqState::Unserved &&
                    state.current_loads[vehicle_id] + self.instance.demands()[req_id] <= self.instance.cap()
                });

                let has_pending_dropoffs = (0..self.instance.n_reqs()).any(|req_id| {
                    state.req_states[req_id] == ReqState::PickedUp &&
                    state.pickup_vehicle[req_id] == Some(vehicle_id)
                });

                // Only return to depot if no more work can be done from current location
                if !has_feasible_pickups && !has_pending_dropoffs {
                    if let Some(new_state) = self.apply_depot_return(state, vehicle_id) {
                        successors.push(new_state);
                    }
                }
            }
        }

        successors
    }

    fn apply_pickup(&self, state: &PartialSolution<'a>, vehicle_id: usize, req_id: usize) -> Option<PartialSolution<'a>> {
        let mut new_state = state.clone();
        let pickup_index = 1 + req_id; // pickup locations start at index 1
        
        new_state.routes[vehicle_id].push(pickup_index);
        new_state.req_states[req_id] = ReqState::PickedUp;
        new_state.current_loads[vehicle_id] += self.instance.demands()[req_id];
        new_state.pickup_vehicle[req_id] = Some(vehicle_id);
        
        Some(new_state)
    }

    fn apply_dropoff(&self, state: &PartialSolution<'a>, vehicle_id: usize, req_id: usize) -> Option<PartialSolution<'a>> {
        let mut new_state = state.clone();
        let dropoff_index = 1 + self.instance.n_reqs() + req_id; // dropoff locations after pickups
        
        new_state.routes[vehicle_id].push(dropoff_index);
        new_state.req_states[req_id] = ReqState::Delivered;
        new_state.current_loads[vehicle_id] -= self.instance.demands()[req_id];
        new_state.served_count += 1;
        
        Some(new_state)
    }

    fn apply_depot_return(&self, state: &PartialSolution<'a>, vehicle_id: usize) -> Option<PartialSolution<'a>> {
        let mut new_state = state.clone();
        new_state.routes[vehicle_id].push(0); // 0 represents depot
        
        Some(new_state)
    }

    fn select_best_states(&self, mut states: Vec<PartialSolution<'a>>, dist_matrix: &[Vec<usize>]) -> Vec<PartialSolution<'a>> {
        if states.len() <= self.beam_width {
            return states;
        }

        // Remove duplicates and invalid states
        states.sort_by(|a, b| {
            let score_a = self.heuristic_score(a, dist_matrix);
            let score_b = self.heuristic_score(b, dist_matrix);
            score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
        });
        states.dedup();

        // Score all states and take best ones
        let mut scored_states: Vec<(f64, PartialSolution<'a>)> = states
            .into_iter()
            .map(|state| (self.heuristic_score(&state, dist_matrix), state))
            .collect();

        scored_states.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        scored_states
            .into_iter()
            .take(self.beam_width)
            .map(|(_, state)| state)
            .collect()
    }

    fn heuristic_score(&self, state: &PartialSolution<'a>, dist_matrix: &[Vec<usize>]) -> f64 {
        let route_distances: Vec<f64> = state.routes.iter()
            .map(|r| self.compute_route_distance(r, dist_matrix))
            .collect();

        let total_distance: f64 = route_distances.iter().sum();
        let fairness = self.compute_jain_fairness(&route_distances);

        let gamma = self.instance.gamma();
        let rho = self.instance.rho();

        let gamma_penalty = if state.served_count < gamma {
            (gamma - state.served_count) as f64 * 10000.0
        } else {
            0.0
        };

        let fairness_penalty = rho * (1.0 - fairness);

        // Heavy penalty for excessive depot returns
        let depot_return_penalty = state.routes.iter()
            .map(|route| {
                route.iter().filter(|&&loc| loc == 0).count() as f64 * 500.0
            })
            .sum::<f64>();

        // Penalty for short routes (encourage serving multiple requests per route)
        let route_efficiency_penalty = state.routes.iter()
            .map(|route| {
                let served_in_route = route.iter().filter(|&&loc| loc > self.instance.n_reqs()).count();
                if served_in_route < 2 && !route.is_empty() {
                    200.0 // Penalty for routes serving too few requests
                } else {
                    0.0
                }
            })
            .sum::<f64>();

        total_distance + fairness_penalty + gamma_penalty + depot_return_penalty + route_efficiency_penalty
    }

    fn compute_route_distance(&self, route: &[usize], dist_matrix: &[Vec<usize>]) -> f64 {
        if route.is_empty() {
            return 0.0;
        }
        
        let mut distance = 0.0;
        
        // From depot to first stop (if first stop is not depot)
        if route[0] != 0 {
            distance += dist_matrix[0][route[0]] as f64;
        }
        
        // Between consecutive stops
        for i in 0..route.len() - 1 {
            distance += dist_matrix[route[i]][route[i + 1]] as f64;
        }
        
        // From last stop back to depot (if last stop is not depot)
        if let Some(&last_stop) = route.last() {
            if last_stop != 0 {
                distance += dist_matrix[last_stop][0] as f64;
            }
        }
        
        distance
    }

    fn compute_jain_fairness(&self, route_distances: &[f64]) -> f64 {
        let sum: f64 = route_distances.iter().sum();
        let sum_sq: f64 = route_distances.iter().map(|d| d * d).sum();
        let k = route_distances.len() as f64;

        if sum_sq == 0.0 {
            1.0
        } else {
            (sum * sum) / (k * sum_sq)
        }
    }

    fn is_feasible(&self, state: &PartialSolution<'a>) -> bool {
        for &load in &state.current_loads {
            if load > self.instance.cap() {
                return false;
            }
        }

        // Check that all delivered requests were properly picked up
        for req_id in 0..self.instance.n_reqs() {
            if state.req_states[req_id] == ReqState::Delivered {
                if state.pickup_vehicle[req_id].is_none() {
                    return false;
                }
            }
        }

        true
    }

    fn best_complete_solution(&self, beam: &[PartialSolution<'a>], dist_matrix: &[Vec<usize>]) -> Option<Solution<'a>> {
        let complete_solutions: Vec<_> = beam.iter()
            .filter(|state| state.served_count >= self.instance.gamma() && self.is_feasible(state))
            .collect();

        if complete_solutions.is_empty() {
            return None;
        }

        // Find the solution with best objective value
        complete_solutions.iter()
            .min_by(|a, b| {
                let sol_a = Solution::new(self.instance, a.routes.clone());
                let sol_b = Solution::new(self.instance, b.routes.clone());
                
                let obj_a = sol_a.objective_function_value();
                let obj_b = sol_b.objective_function_value();
                
                obj_a.partial_cmp(&obj_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|state| Solution::new(self.instance, state.routes.clone()))
    }

    fn fallback_solution(&self) -> Solution<'a> {
        Solution::empty(self.instance, self.instance.n_vehicles())
    }
}

impl<'a> crate::Solver for BeamSearch<'a> {
    fn solve(&self) -> Solution<'a> {
        self.search()
    }
}