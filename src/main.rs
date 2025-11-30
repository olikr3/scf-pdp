use scf_pdp::{BeamSearch, DeterministicConstruction, Instance, RandomConstruction, Solver, LocalSearch, SolverRuntime};
use scf_pdp::local_search::{LocalSearchConfig, Neighborhood};
use scf_pdp::vnd::VND;
use scf_pdp::grasp::{GRASP, GRASPConfig};
use scf_pdp::sim_annealing::{SimulatedAnnealing, SimulatedAnnealingConfig};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy)]
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

    /// Get all available sizes
    fn all() -> Vec<InstanceReqSize> {
        vec![
            InstanceReqSize::Size50,
            InstanceReqSize::Size100,
            InstanceReqSize::Size200,
            InstanceReqSize::Size500,
            InstanceReqSize::Size1000,
            InstanceReqSize::Size2000,
            InstanceReqSize::Size5000,
            InstanceReqSize::Size10000,
        ]
    }

    /// Get small instance sizes (for quick testing)
    fn small() -> Vec<InstanceReqSize> {
        vec![
            InstanceReqSize::Size50,
            InstanceReqSize::Size100,
        ]
    }

    /// Get medium instance sizes
    fn medium() -> Vec<InstanceReqSize> {
        vec![
            InstanceReqSize::Size200,
            InstanceReqSize::Size500,
            InstanceReqSize::Size1000,
        ]
    }

    /// Get large instance sizes
    fn large() -> Vec<InstanceReqSize> {
        vec![
            InstanceReqSize::Size2000,
            InstanceReqSize::Size5000,
            InstanceReqSize::Size10000,
        ]
    }
}

/// Configuration for which solvers to run
#[derive(Debug, Clone)]
pub struct SolverConfig {
    pub run_deterministic: bool,
    pub run_random: bool,
    pub run_beam_search: bool,
    pub run_local_search: bool,
    pub run_vnd: bool,
    pub run_grasp: bool,
    pub run_simulated_annealing: bool,
    pub run_metaheuristic_comparison: bool,
}

impl Default for SolverConfig {
    fn default() -> Self {
        Self {
            run_deterministic: true,
            run_random: false,
            run_beam_search: false,
            run_local_search: false,
            run_vnd: false,
            run_grasp: false,
            run_simulated_annealing: false,
            run_metaheuristic_comparison: false,
        }
    }
}

impl SolverConfig {
    /// Run only fast constructive heuristics
    pub fn fast_only() -> Self {
        Self {
            run_deterministic: true,
            run_random: true,
            ..Default::default()
        }
    }

    /// Run all solvers
    pub fn all() -> Self {
        Self {
            run_deterministic: true,
            run_random: true,
            run_beam_search: true,
            run_local_search: true,
            run_vnd: true,
            run_grasp: true,
            run_simulated_annealing: true,
            run_metaheuristic_comparison: true,
        }
    }

    /// Run only metaheuristics
    pub fn metaheuristics_only() -> Self {
        Self {
            run_vnd: true,
            run_grasp: true,
            run_simulated_annealing: true,
            run_metaheuristic_comparison: true,
            ..Default::default()
        }
    }
}

fn load_instances_from_folder(size: &InstanceReqSize, dataset_type: &str) -> Result<Vec<Instance>, Box<dyn std::error::Error>> {
    let base_path = Path::new("instances");
    let size_folder = size.as_str();
    let dataset_path = base_path.join(size_folder).join(dataset_type);
    
    // Check if the path exists
    if !dataset_path.exists() {
        println!("Path does not exist: {:?}, skipping...", dataset_path);
        return Ok(Vec::new());
    }
    
    let mut instances = Vec::new();
    let entries = fs::read_dir(&dataset_path)?;
    
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        // Only process .txt files
        if path.extension().and_then(|s| s.to_str()) == Some("txt") {
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

fn run_solvers_on_instances(instances: Vec<Instance>, config: &SolverConfig, dataset_type: &str) {
    if instances.is_empty() {
        return;
    }

    let runtime = SolverRuntime::new(instances);
    
    if config.run_deterministic {
        println!("\n=== Running Deterministic Construction ({}) ===", dataset_type);
        let _det_solutions = runtime.run_deterministic();
    }
    
    if config.run_random {
        println!("\n=== Running Random Construction ({}) ===", dataset_type);
        let _rand_solutions = runtime.run_random();
    }
    
    if config.run_beam_search {
        println!("\n=== Running Beam Search ({}) ===", dataset_type);
        let _beam_solutions = runtime.run_beam_search(20, 150);
    }
    
    if config.run_local_search {
        println!("\n=== Running Local Search ({}) ===", dataset_type);
        let _local_solutions = runtime.run_local_search(LocalSearchConfig::default());
    }
    
    if config.run_vnd {
        println!("\n=== Running VND ({}) ===", dataset_type);
        let neighborhoods = vec![
            Neighborhood::Relocate,
            Neighborhood::Exchange,
            Neighborhood::TwoOpt,
        ];
        let _vnd_solutions = runtime.run_vnd(&neighborhoods, 100);
    }
    
    if config.run_grasp {
        println!("\n=== Running GRASP ({}) ===", dataset_type);
        let grasp_config = GRASPConfig::default();
        let _grasp_solutions = runtime.run_grasp(grasp_config);
    }
    
    if config.run_simulated_annealing {
        println!("\n=== Running Simulated Annealing ({}) ===", dataset_type);
        let sa_config = SimulatedAnnealingConfig::default();
        let _sa_solutions = runtime.run_simulated_annealing(sa_config);
    }
    
    if config.run_metaheuristic_comparison {
        println!("\n=== Running Metaheuristic Comparison ({}) ===", dataset_type);
        let _comparison_results = runtime.run_metaheuristic_comparison();
    }
}

fn process_size(size: InstanceReqSize, config: &SolverConfig, process_train: bool, process_test: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{}", "=".repeat(80));
    println!("Processing instances of size: {}", size.as_str());
    println!("{}\n", "=".repeat(80));
    
    // Load instances
    let train_instances = if process_train {
        load_instances_from_folder(&size, "train")?
    } else {
        Vec::new()
    };
    
    let test_instances = if process_test {
        load_instances_from_folder(&size, "test")?
    } else {
        Vec::new()
    };
    
    // Skip if no instances found
    if train_instances.is_empty() && test_instances.is_empty() {
        println!("No instances found for size {}, skipping...\n", size.as_str());
        return Ok(());
    }
    
    // Process train instances if available
    if !train_instances.is_empty() {
        println!("\n--- Processing TRAIN instances (size: {}) ---", size.as_str());
        run_solvers_on_instances(train_instances, config, "train");
    }
    
    // Process test instances if available
    if !test_instances.is_empty() {
        println!("\n--- Processing TEST instances (size: {}) ---", size.as_str());
        run_solvers_on_instances(test_instances, config, "test");
    }
    
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ========== CONFIGURATION ==========
    
    let sizes = InstanceReqSize::small();
    let solver_config = SolverConfig::default();
    
    // Choose whether to process train and/or test sets
    let process_train = true;
    let process_test = false;
    
    // ===================================
    
    println!("Starting batch processing...");
    println!("Sizes to process: {}", sizes.len());
    println!("Solvers enabled:");
    println!("  - Deterministic: {}", solver_config.run_deterministic);
    println!("  - Random: {}", solver_config.run_random);
    println!("  - Beam Search: {}", solver_config.run_beam_search);
    println!("  - Local Search: {}", solver_config.run_local_search);
    println!("  - VND: {}", solver_config.run_vnd);
    println!("  - GRASP: {}", solver_config.run_grasp);
    println!("  - Simulated Annealing: {}", solver_config.run_simulated_annealing);
    println!("  - Metaheuristic Comparison: {}", solver_config.run_metaheuristic_comparison);
    println!();
    
    // Process all sizes
    for size in sizes {
        if let Err(e) = process_size(size, &solver_config, process_train, process_test) {
            eprintln!("Error processing size {}: {}", size.as_str(), e);
            eprintln!("Continuing with next size...\n");
        }
    }
    
    println!("\n{}", "=".repeat(80));
    println!("All instance sizes processed!");
    println!("{}", "=".repeat(80));
    
    Ok(())
}
