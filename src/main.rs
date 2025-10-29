use scf_pdp::{Instance, Solution,};
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

fn load_all_instances(base_path: &str) -> Result<Vec<(String, Instance)>, Box<dyn Error>> {
    let mut instances = Vec::new();
    
    for entry in fs::read_dir(base_path)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            // This should be a request size folder (e.g., "50")
            let req_size = path.file_name().unwrap().to_string_lossy();
            
            // Load test instances
            let test_path = path.join("test");
            if test_path.exists() {
                for test_entry in fs::read_dir(&test_path)? {
                    let test_entry = test_entry?;
                    let test_path = test_entry.path();
                    if test_path.is_file() && test_path.extension().map_or(false, |ext| ext == "txt") {
                        let instance = Instance::from_file(&test_path)?;
                        instances.push((format!("{}/test/{}", req_size, test_path.file_name().unwrap().to_string_lossy()), instance));
                    }
                }
            }
            
            // Load train instances
            let train_path = path.join("train");
            if train_path.exists() {
                for train_entry in fs::read_dir(&train_path)? {
                    let train_entry = train_entry?;
                    let train_path = train_entry.path();
                    if train_path.is_file() && train_path.extension().map_or(false, |ext| ext == "txt") {
                        let instance = Instance::from_file(&train_path)?;
                        instances.push((format!("{}/train/{}", req_size, train_path.file_name().unwrap().to_string_lossy()), instance));
                    }
                }
            }
        }
    }
    
    Ok(instances)
}

fn load_specific_instance(base_path: &str, req_size: InstanceReqSize, dataset: &str, filename: &str) -> Result<Instance, Box<dyn Error>> {
    let path = format!("{}/{}/{}/{}", base_path, req_size.as_str(), dataset, filename);
    Instance::from_file(&path)
}

fn main() -> Result<(), Box<dyn Error>> {
    let base_path = "instances";
    
    // Option 1: Load all instances
    println!("Loading all instances...");
    let all_instances = load_all_instances(base_path)?;
    println!("Loaded {} instances", all_instances.len());
    
    // Print info about each loaded instance
    for (name, instance) in &all_instances {
        println!("{}: {} requests, {} vehicles", name, instance.n_reqs(), instance.n_vehicles());
    }
    /*
    // Option 2: Load a specific instance
    println!("\nLoading specific instance...");
    let specific_instance = load_specific_instance(
        base_path, 
        InstanceReqSize::Size100, 
        "train", 
        "instance_1.txt"  // replace with actual filename
    )?;
    println!("Specific instance: {} requests, {} vehicles", 
             specific_instance.n_reqs(), specific_instance.n_vehicles());
    */
    
    Ok(())
}
