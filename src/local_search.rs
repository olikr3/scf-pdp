use crate::{Instance, Solution, Solver};


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
    pub neighborhood: Neighborhood,
    pub step_function: StepFunction,
    pub acceptance: AcceptanceCriterion,
    pub max_iterations: usize,
    pub max_no_improvement: usize,
    pub time_limit_seconds: u64,
}

impl Default for LocalSearchConfig {
    fn default() -> Self {
        Self {
            neighborhood: Neighborhood::Exchange,
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

impl <'a> LocalSearch<'a> {
    
    fn construct_solution(&self) -> Solution {

        let nborhood = match self.config.neighborhood {
            Neighborhood::Relocate => self.relocate_nh(),
            Neighborhood::Exchange => self.exchange_nh(),
            Neighborhood::TwoOpt => self.two_opt_nh(),
        };

        Solution::empty(self.instance, self.instance.n_vehicles())
    }

    fn relocate_nh(&self) -> Solution {
        todo!()
    }
    fn exchange_nh(&self) -> Solution {
        todo!()
    }
    fn two_opt_nh(&self) -> Solution {
        todo!()
    }
}

impl <'a> Solver for LocalSearch<'a> {
    fn solve(&self) -> Solution {
        self.construct_solution()
    }
}