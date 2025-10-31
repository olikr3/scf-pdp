use crate::{Instance, Solution, solver::construction::ConstructionHeuristic};

/// constructs solution from gamma 
pub struct DeterministicConstruction {
    instance: Instance, 
}

impl DeterministicConstruction {

    fn from_instance(inst: Instance) -> Solution {
        let dist = instance.compute_distance_matrix();

    }
}
