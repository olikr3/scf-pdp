use std::time::Instant;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use crate::{BeamSearch, DeterministicConstruction, Instance, Solution, RandomConstruction, Solver, LocalSearch};
use crate::local_search::LocalSearchConfig;

pub struct SolverRuntime {
    instances: Vec<Instance>,
}

impl SolverRuntime {
    pub fn new(instances: Vec<Instance>) -> Self {
        Self { instances }
    }

    pub fn run_deterministic(&self) -> Vec<Solution> {
        self.run_generic("deterministic", |instance| {
            let solver = DeterministicConstruction::new(instance);
            solver.solve()
        })
    }

    pub fn run_random(&self) -> Vec<Solution> {
        self.run_generic("random", |instance| {
            let solver = RandomConstruction::new(instance, false);
            solver.solve()
        })
    }

    pub fn run_beam_search(&self, beam_width: usize, max_depth: usize) -> Vec<Solution> {
        self.run_generic("beam_search", |instance| {
            let solver = BeamSearch::new(instance.clone())
                .with_beam_width(beam_width)
                .with_max_depth(max_depth);
            solver.solve()
        })
    }

    pub fn run_local_search(&self, config: LocalSearchConfig) -> Vec<Solution> {
        self.run_generic("local_search", |instance| {
            let solver = LocalSearch::new(instance, config.clone());
            solver.solve()
        })
    }

    fn run_generic<F>(&self, solver_name: &str, solver_fn: F) -> Vec<Solution> 
    where 
        F: Fn(&Instance) -> Solution,
    {
        let mut solutions = Vec::new();
        let mut csv_data = Vec::new();
        
        if let Some(first_instance) = self.instances.first() {
            let instance_size = first_instance.n_reqs().to_string();
            let results_dir = format!("results/{}", instance_size);
            fs::create_dir_all(&results_dir).expect("Failed to create results directory");
            
            csv_data.push("instance_name,time_seconds,objective_value,jain_fairness,num_vehicles".to_string());
        }
        
        for instance in &self.instances {
            println!("Solving instance: {}", instance.name());
            
            let start_time = Instant::now();
            let solution = solver_fn(instance);
            let duration = start_time.elapsed();
            let time_seconds = duration.as_secs_f64();
            
            println!("  Solved in {:.2?}", duration);
            println!("  Objective value: {:.2}", solution.objective_function_value());
            println!("  Total distance: {:.2}", solution.total_travel_distance());
            println!("  Jain fairness: {:.4}", solution.jain_fairness());
            println!("  Valid solution: {}", solution.is_valid());
            println!();
            
            // Add CSV row
            let csv_row = format!(
                "{},{:.6},{:.6},{:.6},{}",
                instance.name(),
                time_seconds,
                solution.objective_function_value(),
                solution.jain_fairness(),
                solution.routes.len()
            );
            csv_data.push(csv_row);
            
            solutions.push(solution);
        }
        
        // Write CSV file
        if let Some(first_instance) = self.instances.first() {
            let instance_size = first_instance.n_reqs().to_string();
            let csv_filename = format!("results/{}/{}.csv", instance_size, solver_name);
            
            let mut file = File::create(&csv_filename).expect("Failed to create CSV file");
            for line in csv_data {
                writeln!(file, "{}", line).expect("Failed to write to CSV file");
            }
            println!("Results written to: {}", csv_filename);
        }
        
        solutions
    }

    pub fn run_comparison(&self) -> Vec<(String, Solution, Solution, Solution, Solution)> {
        let mut results = Vec::new();
        let mut csv_data = Vec::new();
        
        // Create results directory structure based on instance size
        if let Some(first_instance) = self.instances.first() {
            let instance_size = first_instance.n_reqs().to_string();
            let results_dir = format!("results/{}", instance_size);
            fs::create_dir_all(&results_dir).expect("Failed to create results directory");
            
            // Add CSV header for comparison
            csv_data.push("instance_name,det_time,det_objective,det_fairness,det_vehicles,rand_time,rand_objective,rand_fairness,rand_vehicles,beam_time,beam_objective,beam_fairness,beam_vehicles,local_time,local_objective,local_fairness,local_vehicles".to_string());
        }
        
        for instance in &self.instances {
            println!("Comparing solvers for instance: {}", instance.name());
            
            let det_start = Instant::now();
            let det_solver = DeterministicConstruction::new(instance);
            let det_solution = det_solver.solve();
            let det_time = det_start.elapsed().as_secs_f64();
            
            let rand_start = Instant::now();
            let rand_solver = RandomConstruction::new(instance, false);
            let rand_solution = rand_solver.solve();
            let rand_time = rand_start.elapsed().as_secs_f64();
            
            let beam_start = Instant::now();
            let beam_solver = BeamSearch::new(instance.clone()).with_beam_width(20).with_max_depth(150);
            let beam_solution = beam_solver.solve();
            let beam_time = beam_start.elapsed().as_secs_f64();
            
            let local_start = Instant::now();
            let local_solver = LocalSearch::new(instance, LocalSearchConfig::default());
            let local_solution = local_solver.solve();
            let local_time = local_start.elapsed().as_secs_f64();
            
            // Add CSV row for comparison
            let csv_row = format!(
                "{},{:.6},{:.6},{:.6},{},{:.6},{:.6},{:.6},{},{:.6},{:.6},{:.6},{},{:.6},{:.6},{:.6},{}",
                instance.name(),
                det_time,
                det_solution.objective_function_value(),
                det_solution.jain_fairness(),
                det_solution.routes.len(), // Use routes.len() instead of num_vehicles()
                rand_time,
                rand_solution.objective_function_value(),
                rand_solution.jain_fairness(),
                rand_solution.routes.len(), // Use routes.len() instead of num_vehicles()
                beam_time,
                beam_solution.objective_function_value(),
                beam_solution.jain_fairness(),
                beam_solution.routes.len(), // Use routes.len() instead of num_vehicles()
                local_time,
                local_solution.objective_function_value(),
                local_solution.jain_fairness(),
                local_solution.routes.len() // Use routes.len() instead of num_vehicles()
            );
            csv_data.push(csv_row);
            
            results.push((instance.name().to_string(), det_solution, rand_solution, beam_solution, local_solution));
        }
        
        // Write comparison CSV file
        if let Some(first_instance) = self.instances.first() {
            let instance_size = first_instance.n_reqs().to_string();
            let csv_filename = format!("results/{}/comparison.csv", instance_size);
            
            let mut file = File::create(&csv_filename).expect("Failed to create comparison CSV file");
            for line in csv_data {
                writeln!(file, "{}", line).expect("Failed to write to comparison CSV file");
            }
            println!("Comparison results written to: {}", csv_filename);
        }
        
        results
    }
}