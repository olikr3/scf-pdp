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


fn main () {
    let curr_size_dir_name = InstanceReqSize::Size50;
    /*
    need to to something like:
    for file in filepath.files:
        curr_file = filepath + curr.file
        inst = Instance.from_file();
        det_const = DeterministicConstruction.from_instance()
    */
    // let inst = Instance.from_file();
}
