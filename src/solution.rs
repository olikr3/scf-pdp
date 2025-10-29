#[derive(Debug, Clone)]
pub struct Solution {
    pub solution_name: String, // denotes the kind of heuristic used
    pub routes: Vec<Vec<usize>>, // vehicle routes (indices into locations)
}

impl Solution {
}