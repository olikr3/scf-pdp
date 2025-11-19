use scf_pdp::{BeamSearch, DeterministicConstruction, Instance, RandomConstruction, Solver, LocalSearch, SolverRuntime};
use scf_pdp::local_search::LocalSearchConfig;
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub enum InstanceReqSize {
    Size50,
    Size100,
    Size200,
    Size500,
    Size1000,
    Size2000,
    Size5000,
    Size10000,
}

impl InstanceReqSize {
    fn as_str(&self) -> &str {
        match self {
            InstanceReqSize::Size50 => "50",
            InstanceReqSize::Size100 => "100",
            InstanceReqSize::Size200 => "200",
            InstanceReqSize::Size500 => "500",
            InstanceReqSize::Size1000 => "1000",
            InstanceReqSize::Size2000 => "2000",
            InstanceReqSize::Size5000 => "5000",
            InstanceReqSize::Size10000 => "10000",
        }
    }
}

fn load_instances_from_folder(size: &InstanceReqSize, dataset_type: &str) -> Result<Vec<Instance>, Box<dyn std::error::Error>> {
    let base_path = Path::new("instances");
    let size_folder = size.as_str();
    let dataset_path = base_path.join(size_folder).join(dataset_type);
    
    let mut instances = Vec::new();
    let entries = fs::read_dir(&dataset_path)?;
    
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        // Only process .txt files
        if path.extension().and_then(|s| s.to_str()) == Some("txt") {
            //println!("Loading instance: {:?}", path);
            
            match Instance::from_file(path.to_str().unwrap()) {
                Ok(instance) => {
                    instances.push(instance);
                }
                Err(e) => {
                    eprintln!("Failed to load instance {:?}: {}", path, e);
                }
            }
        }
    }
    
    println!("Loaded {} instances from {:?}", instances.len(), dataset_path);
    Ok(instances)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let size = InstanceReqSize::Size50;
    let train_instances = load_instances_from_folder(&size, "train")?;
    let test_instances = load_instances_from_folder(&size, "test")?;

    // Create runtime
    let runtime = SolverRuntime::new(train_instances);

    // Run individual solvers
    println!("=== Running Deterministic Construction ===");
    let det_solutions = runtime.run_deterministic();

    println!("=== Running Random Construction ===");
    let rand_solutions = runtime.run_random();

    println!("=== Running Beam Search ===");
    let beam_solutions = runtime.run_beam_search(20, 150);

    println!("=== Running Local Search ===");
    let local_solutions = runtime.run_local_search(LocalSearchConfig::default());

    // Or run comparison
    println!("=== Running Solver Comparison ===");
    let comparison_results = runtime.run_comparison();
    
    for (instance_name, det_sol, rand_sol, beam_sol, local_sol) in comparison_results {
        println!("Instance: {}", instance_name);
        println!("  Deterministic: {:.2} (valid: {})", det_sol.objective_function_value(), det_sol.is_valid());
        println!("  Random: {:.2} (valid: {})", rand_sol.objective_function_value(), rand_sol.is_valid());
        println!("  Beam Search: {:.2} (valid: {})", beam_sol.objective_function_value(), beam_sol.is_valid());
        println!("  Local Search: {:.2} (valid: {})", local_sol.objective_function_value(), local_sol.is_valid());
        println!();
    }

    Ok(())
}
