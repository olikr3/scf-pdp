use crate::{Solution};

pub trait Solver {
    /// Solve the instance and return a solution
    fn solve(&self) -> Solution;
}