use crate::{Instance, LocalSearch, LocalSearchConfig, Neighborhood, RandomConstruction, Solution, Solver, StepFunction, AcceptanceCriterion};

/// Variable Neighborhood Descent solver
/// Systematically explores different neighborhoods, returning to the first
/// whenever an improvement is found
pub struct VND<'a> {
    instance: &'a Instance,
    neighborhoods: Vec<Neighborhood>,
    max_iterations: usize,
    biased_construction: bool,
}

impl<'a> VND<'a> {
    pub fn new(instance: &'a Instance, neighborhoods: Vec<Neighborhood>) -> Self {
        Self {
            instance,
            neighborhoods,
            max_iterations: 1000,
            biased_construction: true,
        }
    }

    pub fn with_max_iterations(mut self, max_iterations: usize) -> Self {
        self.max_iterations = max_iterations;
        self
    }

    pub fn with_biased_construction(mut self, biased: bool) -> Self {
        self.biased_construction = biased;
        self
    }

    fn construct_initial_solution(&self) -> Solution {
        let random_construction = RandomConstruction::new(self.instance, self.biased_construction);
        random_construction.solve()
    }

    fn explore_neighborhood(&self, current: &Solution, neighborhood: Neighborhood) -> Option<Solution> {
        // Create a local search config for this specific neighborhood
        let config = LocalSearchConfig {
            neighborhood,
            step_function: StepFunction::FirstImprovement,
            acceptance: AcceptanceCriterion::ImprovingOnly,
            max_iterations: 1, // Only one iteration to find best neighbor
            max_no_improvement: 1,
            time_limit_seconds: 50,
        };

        let local_search = LocalSearch::new(self.instance, config);
        
        // Generate all neighbors in this neighborhood
        let neighbors = local_search.generate_neighbors(current);
        
        if neighbors.is_empty() {
            return None;
        }

        // Find the best improving neighbor
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
}

impl<'a> Solver for VND<'a> {
    fn solve(&self) -> Solution {
        // Start with an initial solution
        let mut current = self.construct_initial_solution();
        let mut best_solution = current.clone();
        let mut best_obj = current.objective_function_value();

        let mut iterations = 0;

        // VND main loop
        while iterations < self.max_iterations {
            let mut improved = false;
            
            // Try each neighborhood in sequence
            for &neighborhood in &self.neighborhoods {
                if let Some(better_solution) = self.explore_neighborhood(&current, neighborhood) {
                    let better_obj = better_solution.objective_function_value();
                    
                    // Accept the improvement
                    current = better_solution;
                    
                    // Update best solution if necessary
                    if better_obj < best_obj {
                        best_solution = current.clone();
                        best_obj = better_obj;
                    }
                    
                    improved = true;
                    // Reset to first neighborhood when improvement found
                    break;
                }
            }

            // If no improvement found in any neighborhood, we're at a local optimum
            if !improved {
                break;
            }

            iterations += 1;
        }

        best_solution
    }
}