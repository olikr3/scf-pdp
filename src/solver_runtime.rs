use std::time::Instant;
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
        self.run_generic(|instance| {
            let solver = DeterministicConstruction::new(instance);
            solver.solve()
        })
    }

    pub fn run_random(&self) -> Vec<Solution> {
        self.run_generic(|instance| {
            let solver = RandomConstruction::new(instance, false);
            solver.solve()
        })
    }

    pub fn run_beam_search(&self, beam_width: usize, max_depth: usize) -> Vec<Solution> {
        self.run_generic(|instance| {
            let solver = BeamSearch::new(instance.clone())
                .with_beam_width(beam_width)
                .with_max_depth(max_depth);
            solver.solve()
        })
    }

    pub fn run_local_search(&self, config: LocalSearchConfig) -> Vec<Solution> {
        self.run_generic(|instance| {
            let solver = LocalSearch::new(instance, config.clone());
            solver.solve()
        })
    }

    fn run_generic<F>(&self, solver_fn: F) -> Vec<Solution> 
    where 
        F: Fn(&Instance) -> Solution,
    {
        let mut solutions = Vec::new();
        
        for instance in &self.instances {
            println!("Solving instance: {}", instance.name());
            
            let start_time = Instant::now();
            let solution = solver_fn(instance);
            let duration = start_time.elapsed();
            
            println!("  Solved in {:.2?}", duration);
            println!("  Objective value: {:.2}", solution.objective_function_value());
            println!("  Total distance: {:.2}", solution.total_travel_distance());
            println!("  Jain fairness: {:.4}", solution.jain_fairness());
            println!("  Valid solution: {}", solution.is_valid());
            println!();
            
            solutions.push(solution);
        }
        
        solutions
    }

    pub fn run_comparison(&self) -> Vec<(String, Solution, Solution, Solution, Solution)> {
        let mut results = Vec::new();
        
        for instance in &self.instances {
            println!("Comparing solvers for instance: {}", instance.name());
            
            let det_solver = DeterministicConstruction::new(instance);
            let rand_solver = RandomConstruction::new(instance, false);
            let beam_solver = BeamSearch::new(instance.clone()).with_beam_width(20).with_max_depth(150);
            let local_solver = LocalSearch::new(instance, LocalSearchConfig::default());
            
            let det_solution = det_solver.solve();
            let rand_solution = rand_solver.solve();
            let beam_solution = beam_solver.solve();
            let local_solution = local_solver.solve();
            
            results.push((instance.name().to_string(), det_solution, rand_solution, beam_solution, local_solution));
        }
        
        results
    }
}