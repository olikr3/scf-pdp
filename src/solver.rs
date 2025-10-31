pub trait Solver {
    /// Solve the instance and return a solution
    fn solve(&self) -> Solution;
    
    /// Get the name of the solver
    fn name(&self) -> &str;
}