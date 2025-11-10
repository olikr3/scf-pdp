use crate::{Instance, Solution, Solver};

/*
Samples gamma requests at random and assigns them to vehicles uniformally random if they have spare capacity.
Can be biased towards routes closer to the depot.
*/
pub struct RandomConstruction {
    instance: Instance,
    biased: bool,
}


impl RandomConstruction {
    
    fn new(instance: Instance, biased: bool) -> Self {
        Self { instance, biased }
    }

    fn construct_solution(&self) -> Solution {
    
        todo!()
    }
}

impl Solver for RandomConstruction {
    fn solve(&self) -> Solution {
        self.construct_solution()
    }
}