use scf_pdp::{BeamSearch, DeterministicConstruction, Instance, RandomConstruction, Solver};
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
    let train_instances = load_instances_from_folder(&size, "train")?; // note that calling "cargo run" in src does not work - needs to be called in root directory
    let _test_instances = load_instances_from_folder(&size, "test")?;

    for i in 0..train_instances.len() {
        let current_inst = &train_instances[i];
        let det_solver = DeterministicConstruction::new(current_inst);
        let rand_solver = RandomConstruction::new(current_inst, false);
        let soln = det_solver.solve();
        let soln1 = rand_solver.solve();
        //let soln1 = det_solver.utility_based_construction();
        
        // For BeamSearch, you'll need to clone the instance since BeamSearch now takes ownership
        let beam_search = BeamSearch::new(current_inst.clone())
            .with_beam_width(20)
            .with_max_depth(150);
        let soln2 = beam_search.solve();
        
        println!("Deterministic Solution: {}", soln);
        println!("Random Solution: {}", soln1);
        println!("Beam Search Solution: {}", soln2);
    }
    
    Ok(())
}
