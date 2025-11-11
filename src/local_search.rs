use crate::{Instance, Solution};


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Neighborhood {
    Relocate,    // Move a request from one route to another
    Exchange,    // Swap two requests between routes
    TwoOpt,      // Reverse a segment within a route
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StepFunction {
    FirstImprovement,
    BestImprovement,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AcceptanceCriterion {
    ImprovingOnly,
}

#[derive(Debug, Clone)]
pub struct LocalSearchConfig {
    pub neighborhoods: Vec<Neighborhood>,
    pub step_function: StepFunction,
    pub acceptance: AcceptanceCriterion,
    pub max_iterations: usize,
    pub max_no_improvement: usize,
    pub time_limit_seconds: u64,
}

impl Default for LocalSearchConfig {
    fn default() -> Self {
        Self {
            neighborhoods: vec![
                Neighborhood::Relocate,
                Neighborhood::Exchange,
                Neighborhood::TwoOpt,
            ],
            step_function: StepFunction::FirstImprovement,
            acceptance: AcceptanceCriterion::ImprovingOnly,
            max_iterations: 1000,
            max_no_improvement: 100,
            time_limit_seconds: 60,
        }
    }
}

pub struct LocalSearch<'a> {
    instance: &'a Instance,
    config: LocalSearchConfig,
}