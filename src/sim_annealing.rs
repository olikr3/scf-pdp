use std::time::Instant;
use std::cell::RefCell;
use rand::prelude::*;
use crate::{Instance, LocalSearch, LocalSearchConfig, Neighborhood, RandomConstruction, Solution, Solver, StepFunction, AcceptanceCriterion};

/// Cooling schedule for Simulated Annealing
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CoolingSchedule {
    /// Geometric cooling: T_new = alpha * T_old
    Geometric { alpha: f64 },
    /// Linear cooling: T_new = T_old - beta
    Linear { beta: f64 },
    /// Exponential cooling: T_new = T_0 * alpha^iteration
    Exponential { alpha: f64 },
    /// Logarithmic cooling: T_new = T_0 / (1 + alpha * ln(1 + iteration))
    Logarithmic { alpha: f64 },
}

impl CoolingSchedule {
    pub fn update_temperature(&self, current_temp: f64, initial_temp: f64, iteration: usize) -> f64 {
        match self {
            CoolingSchedule::Geometric { alpha } => current_temp * alpha,
            CoolingSchedule::Linear { beta } => (current_temp - beta).max(0.0),
            CoolingSchedule::Exponential { alpha } => initial_temp * alpha.powi(iteration as i32),
            CoolingSchedule::Logarithmic { alpha } => {
                initial_temp / (1.0 + alpha * ((1 + iteration) as f64).ln())
            }
        }
    }
}

/// Configuration for Simulated Annealing
#[derive(Debug, Clone)]
pub struct SimulatedAnnealingConfig {
    pub initial_temperature: f64,
    pub final_temperature: f64,
    pub cooling_schedule: CoolingSchedule,
    pub max_iterations: usize,
    pub iterations_per_temperature: usize,
    pub neighborhood: Neighborhood,
    pub time_limit_seconds: u64,
    pub biased_construction: bool,
}

impl Default for SimulatedAnnealingConfig {
    fn default() -> Self {
        Self {
            initial_temperature: 1000.0,
            final_temperature: 0.1,
            cooling_schedule: CoolingSchedule::Geometric { alpha: 0.95 },
            max_iterations: 10000,
            iterations_per_temperature: 100,
            neighborhood: Neighborhood::Exchange,
            time_limit_seconds: 300,
            biased_construction: true,
        }
    }
}

/// Simulated Annealing solver for SCF-PDP
pub struct SimulatedAnnealing<'a> {
    instance: &'a Instance,
    config: SimulatedAnnealingConfig,
    rng: RefCell<ThreadRng>,
}

impl<'a> SimulatedAnnealing<'a> {
    pub fn new(instance: &'a Instance, config: SimulatedAnnealingConfig) -> Self {
        Self {
            instance,
            config,
            rng: RefCell::new(thread_rng()),
        }
    }

    /// Construct initial solution
    fn construct_initial_solution(&self) -> Solution {
        let random_construction = RandomConstruction::new(self.instance, self.config.biased_construction);
        random_construction.solve()
    }

    /// Generate a random neighbor from current solution
    fn generate_random_neighbor(&self, current: &Solution) -> Option<Solution> {
        let local_search_config = LocalSearchConfig {
            neighborhood: self.config.neighborhood,
            step_function: StepFunction::FirstImprovement,
            acceptance: AcceptanceCriterion::ImprovingOnly,
            max_iterations: 1,
            max_no_improvement: 1,
            time_limit_seconds: 60,
        };

        let local_search = LocalSearch::new(self.instance, local_search_config);
        let neighbors = local_search.generate_neighbors(current);

        if neighbors.is_empty() {
            return None;
        }

        // Select a random neighbor
        let idx = self.rng.borrow_mut().gen_range(0..neighbors.len());
        Some(neighbors[idx].clone())
    }

    /// Acceptance probability based on Metropolis criterion
    fn acceptance_probability(&self, current_obj: f64, neighbor_obj: f64, temperature: f64) -> f64 {
        if neighbor_obj < current_obj {
            // Always accept improvements
            1.0
        } else {
            // Accept worse solutions with probability exp(-delta/T)
            let delta = neighbor_obj - current_obj;
            (-delta / temperature).exp()
        }
    }

    /// Decide whether to accept a solution
    fn accept_solution(&self, current_obj: f64, neighbor_obj: f64, temperature: f64) -> bool {
        let acceptance_prob = self.acceptance_probability(current_obj, neighbor_obj, temperature);
        let random_value: f64 = self.rng.borrow_mut().gen();
        random_value < acceptance_prob
    }

    /// Calculate initial temperature based on solution quality variance (optional advanced feature)
    pub fn auto_initial_temperature(&self, sample_size: usize) -> f64 {
        let mut objectives = Vec::new();
        
        for _ in 0..sample_size {
            let solution = self.construct_initial_solution();
            if solution.is_valid() {
                objectives.push(solution.objective_function_value());
            }
        }

        if objectives.is_empty() {
            return self.config.initial_temperature;
        }

        // Calculate standard deviation
        let mean: f64 = objectives.iter().sum::<f64>() / objectives.len() as f64;
        let variance: f64 = objectives.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / objectives.len() as f64;
        let std_dev = variance.sqrt();

        // Set initial temperature to 2 * std_dev (a heuristic)
        std_dev * 2.0
    }
}

impl<'a> Solver for SimulatedAnnealing<'a> {
    fn solve(&self) -> Solution {
        let start_time = Instant::now();
        
        // Initialize with a random solution
        let mut current = self.construct_initial_solution();
        let mut best_solution = current.clone();
        let mut best_obj = current.objective_function_value();
        
        let mut temperature = self.config.initial_temperature;
        let initial_temperature = self.config.initial_temperature;
        let mut iteration = 0;
        let mut temp_iteration = 0;

        // Statistics tracking (prefix with _ to avoid unused warnings)
        let mut _accepted_count = 0;
        let mut _rejected_count = 0;

        while temperature > self.config.final_temperature
            && iteration < self.config.max_iterations
            && start_time.elapsed().as_secs() < self.config.time_limit_seconds {
            
            // Perform iterations at current temperature
            for _ in 0..self.config.iterations_per_temperature {
                if let Some(neighbor) = self.generate_random_neighbor(&current) {
                    let current_obj = current.objective_function_value();
                    let neighbor_obj = neighbor.objective_function_value();

                    // Decide whether to accept the neighbor
                    if self.accept_solution(current_obj, neighbor_obj, temperature) {
                        current = neighbor;
                        _accepted_count += 1;

                        // Update best solution if necessary
                        if current_obj < best_obj {
                            best_solution = current.clone();
                            best_obj = current_obj;
                        }
                    } else {
                        _rejected_count += 1;
                    }
                }

                iteration += 1;
                
                // Check termination conditions
                if iteration >= self.config.max_iterations 
                    || start_time.elapsed().as_secs() >= self.config.time_limit_seconds {
                    break;
                }
            }

            // Cool down the temperature
            temp_iteration += 1;
            temperature = self.config.cooling_schedule.update_temperature(
                temperature,
                initial_temperature,
                temp_iteration
            );
        }

        best_solution
    }
}

/// Builder for SimulatedAnnealing to make configuration easier
pub struct SimulatedAnnealingBuilder<'a> {
    instance: &'a Instance,
    config: SimulatedAnnealingConfig,
}

impl<'a> SimulatedAnnealingBuilder<'a> {
    pub fn new(instance: &'a Instance) -> Self {
        Self {
            instance,
            config: SimulatedAnnealingConfig::default(),
        }
    }

    pub fn initial_temperature(mut self, temp: f64) -> Self {
        self.config.initial_temperature = temp;
        self
    }

    pub fn final_temperature(mut self, temp: f64) -> Self {
        self.config.final_temperature = temp;
        self
    }

    pub fn cooling_schedule(mut self, schedule: CoolingSchedule) -> Self {
        self.config.cooling_schedule = schedule;
        self
    }

    pub fn max_iterations(mut self, iterations: usize) -> Self {
        self.config.max_iterations = iterations;
        self
    }

    pub fn iterations_per_temperature(mut self, iterations: usize) -> Self {
        self.config.iterations_per_temperature = iterations;
        self
    }

    pub fn neighborhood(mut self, neighborhood: Neighborhood) -> Self {
        self.config.neighborhood = neighborhood;
        self
    }

    pub fn time_limit_seconds(mut self, seconds: u64) -> Self {
        self.config.time_limit_seconds = seconds;
        self
    }

    pub fn biased_construction(mut self, biased: bool) -> Self {
        self.config.biased_construction = biased;
        self
    }

    pub fn build(self) -> SimulatedAnnealing<'a> {
        SimulatedAnnealing::new(self.instance, self.config)
    }
}