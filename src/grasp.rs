use std::time::Instant;
use crate::{Instance, LocalSearch, LocalSearchConfig, Neighborhood, RandomConstruction, Solution, Solver, StepFunction, AcceptanceCriterion, VND};

/// Greedy Randomized Adaptive Search Procedure (GRASP)
/// Combines randomized construction with local search improvement
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LocalSearchStrategy {
    /// Use a single neighborhood with best improvement
    SingleNeighborhood(Neighborhood),
    /// Use VND with multiple neighborhoods
    VND,
    /// Use a composite neighborhood (union of all neighborhoods)
    CompositeNeighborhood,
}

#[derive(Debug, Clone, Copy)]
pub struct GRASPConfig {
    pub max_iterations: usize,
    pub time_limit_seconds: u64,
    pub local_search_strategy: LocalSearchStrategy,
    pub biased_construction: bool,
    pub local_search_max_iterations: usize,
    pub local_search_time_limit: u64,
}

impl Default for GRASPConfig {
    fn default() -> Self {
        Self {
            max_iterations: 100,
            time_limit_seconds: 300,
            local_search_strategy: LocalSearchStrategy::VND,
            biased_construction: true,
            local_search_max_iterations: 100,
            local_search_time_limit: 60,
        }
    }
}

pub struct GRASP<'a> {
    instance: &'a Instance,
    config: GRASPConfig,
}

impl<'a> GRASP<'a> {
    pub fn new(instance: &'a Instance, config: GRASPConfig) -> Self {
        Self { instance, config }
    }

    /// Construction phase: Generate a randomized solution
    fn construct_solution(&self) -> Solution {
        let random_construction = RandomConstruction::new(self.instance, self.config.biased_construction);
        random_construction.solve()
    }

    /// Local search phase: Improve the solution using the configured strategy
    fn local_search(&self, solution: Solution) -> Solution {
        match self.config.local_search_strategy {
            LocalSearchStrategy::SingleNeighborhood(neighborhood) => {
                self.single_neighborhood_search(solution, neighborhood)
            }
            LocalSearchStrategy::VND => {
                self.vnd_search(solution)
            }
            LocalSearchStrategy::CompositeNeighborhood => {
                self.composite_neighborhood_search(solution)
            }
        }
    }

    /// Local search using a single neighborhood structure
    fn single_neighborhood_search(&self, solution: Solution, neighborhood: Neighborhood) -> Solution {
        let config = LocalSearchConfig {
            neighborhood,
            step_function: StepFunction::BestImprovement,
            acceptance: AcceptanceCriterion::ImprovingOnly,
            max_iterations: self.config.local_search_max_iterations,
            max_no_improvement: 50,
            time_limit_seconds: self.config.local_search_time_limit,
        };

        let local_search = LocalSearch::new(self.instance, config);
        
        let mut current = solution;
        let mut improved = true;

        while improved {
            improved = false;
            let neighbors = local_search.generate_neighbors(&current);
            
            if neighbors.is_empty() {
                break;
            }

            let current_obj = current.objective_function_value();
            let mut best_neighbor: Option<Solution> = None;
            let mut best_obj = current_obj;

            for neighbor in neighbors {
                let neighbor_obj = neighbor.objective_function_value();
                if neighbor_obj < best_obj {
                    best_obj = neighbor_obj;
                    best_neighbor = Some(neighbor);
                }
            }

            if let Some(better) = best_neighbor {
                current = better;
                improved = true;
            }
        }

        current
    }

    /// Local search using Variable Neighborhood Descent
    fn vnd_search(&self, solution: Solution) -> Solution {
        // Use all three neighborhoods in VND
        let neighborhoods = vec![
            Neighborhood::Relocate,
            Neighborhood::Exchange,
            Neighborhood::TwoOpt,
        ];

        let vnd = VND::new(self.instance, neighborhoods.clone())
            .with_max_iterations(self.config.local_search_max_iterations)
            .with_biased_construction(false); // Start from provided solution

        // Since VND constructs its own initial solution, we need to improve the given solution
        // We'll use a modified approach
        self.vnd_search_with_initial(solution, neighborhoods)
    }

    /// VND search starting from a given solution
    fn vnd_search_with_initial(&self, mut current: Solution, neighborhoods: Vec<Neighborhood>) -> Solution {
        let mut best_solution = current.clone();
        let mut best_obj = current.objective_function_value();
        let mut iterations = 0;

        while iterations < self.config.local_search_max_iterations {
            let mut improved = false;
            
            // Try each neighborhood in sequence
            for &neighborhood in &neighborhoods {
                if let Some(better_solution) = self.explore_neighborhood(&current, neighborhood) {
                    let better_obj = better_solution.objective_function_value();
                    
                    current = better_solution;
                    
                    if better_obj < best_obj {
                        best_solution = current.clone();
                        best_obj = better_obj;
                    }
                    
                    improved = true;
                    // Reset to first neighborhood when improvement found
                    break;
                }
            }

            if !improved {
                break;
            }

            iterations += 1;
        }

        best_solution
    }

    /// Explore a single neighborhood and return best improving solution
    fn explore_neighborhood(&self, current: &Solution, neighborhood: Neighborhood) -> Option<Solution> {
        let config = LocalSearchConfig {
            neighborhood,
            step_function: StepFunction::BestImprovement,
            acceptance: AcceptanceCriterion::ImprovingOnly,
            max_iterations: 1,
            max_no_improvement: 1,
            time_limit_seconds: self.config.local_search_time_limit,
        };

        let local_search = LocalSearch::new(self.instance, config);
        let neighbors = local_search.generate_neighbors(current);
        
        if neighbors.is_empty() {
            return None;
        }

        let current_obj = current.objective_function_value();
        let mut best_neighbor: Option<Solution> = None;
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

    /// Local search using composite neighborhood (union of all neighborhoods)
    fn composite_neighborhood_search(&self, solution: Solution) -> Solution {
        let mut current = solution;
        let mut improved = true;

        let neighborhoods = vec![
            Neighborhood::Relocate,
            Neighborhood::Exchange,
            Neighborhood::TwoOpt,
        ];

        while improved {
            improved = false;
            
            // Generate neighbors from ALL neighborhoods (composite)
            let mut all_neighbors = Vec::new();
            
            for &neighborhood in &neighborhoods {
                let config = LocalSearchConfig {
                    neighborhood,
                    step_function: StepFunction::BestImprovement,
                    acceptance: AcceptanceCriterion::ImprovingOnly,
                    max_iterations: 1,
                    max_no_improvement: 1,
                    time_limit_seconds: self.config.local_search_time_limit,
                };

                let local_search = LocalSearch::new(self.instance, config);
                let neighbors = local_search.generate_neighbors(&current);
                all_neighbors.extend(neighbors);
            }

            if all_neighbors.is_empty() {
                break;
            }

            // Find best neighbor across ALL neighborhoods
            let current_obj = current.objective_function_value();
            let mut best_neighbor: Option<Solution> = None;
            let mut best_obj = current_obj;

            for neighbor in all_neighbors {
                let neighbor_obj = neighbor.objective_function_value();
                if neighbor_obj < best_obj {
                    best_obj = neighbor_obj;
                    best_neighbor = Some(neighbor);
                }
            }

            if let Some(better) = best_neighbor {
                current = better;
                improved = true;
            }
        }

        current
    }
}

impl<'a> Solver for GRASP<'a> {
    fn solve(&self) -> Solution {
        let start_time = Instant::now();
        
        let mut best_solution: Option<Solution> = None;
        let mut best_obj = f64::INFINITY;
        
        let mut iteration = 0;

        while iteration < self.config.max_iterations 
            && start_time.elapsed().as_secs() < self.config.time_limit_seconds {
            
            // Construction phase: Generate randomized solution
            let initial_solution = self.construct_solution();
            
            // Skip invalid solutions
            if !initial_solution.is_valid() {
                iteration += 1;
                continue;
            }

            // Local search phase: Improve the solution
            let improved_solution = self.local_search(initial_solution);
            
            // Update best solution if necessary
            if improved_solution.is_valid() {
                let obj = improved_solution.objective_function_value();
                if obj < best_obj {
                    best_obj = obj;
                    best_solution = Some(improved_solution);
                }
            }

            iteration += 1;
        }

        // Return best solution found, or construct a fallback if none found
        best_solution.unwrap_or_else(|| {
            let fallback = RandomConstruction::new(self.instance, true);
            fallback.solve()
        })
    }
}