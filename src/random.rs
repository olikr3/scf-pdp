use crate::{Instance, Solution, Solver};


pub struct RandomConstruction {
    instance: Instance,
}


impl RandomConstruction {
    
    fn new(instance: Instance) -> Self {
        Self { instance }
    }

    fn construct_solution(&self) -> Solution {
        todo!()
    }
}

impl Solver for RandomConstruction {
    fn solve(&self) -> Solution {
        self.construct_solution()
    }

    fn name(&self) -> &str {
        "Random Solution"
    }
}