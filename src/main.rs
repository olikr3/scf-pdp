use scf_pdp::{Instance, Solution};
use std::error::Error;
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
    
    // Read all files in the directory
    let entries = fs::read_dir(&dataset_path)?;
    
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        // Only process .txt files
        if path.extension().and_then(|s| s.to_str()) == Some("txt") {
            println!("Loading instance: {:?}", path);
            
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
    
    println!("Processing size: {}", size.as_str());
    
    let train_instances = load_instances_from_folder(&size, "train")?;
    let test_instances = load_instances_from_folder(&size, "test")?;
    
    // training instances
    for (i, instance) in train_instances.iter().enumerate() {
        println!("=== Training Instance {} ===", i);
        println!("{}", instance);
        
        println!("Instance name: {}", instance.name());
    }
    
    // test instances
    for (i, instance) in test_instances.iter().enumerate() {
        println!("Processing test instance {}: name={}, n_reqs={}, n_vehicles={}", 
                i, instance.name(), instance.n_reqs(), instance.n_vehicles());
        
        //let solution = DeterministicConstruction::from_instance(instance);
    }
    
    Ok(())
}
