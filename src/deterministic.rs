use crate::{Instance, Solution, Solver};

pub struct DeterministicConstruction<'a> {
    instance: &'a Instance,
}

impl<'a> DeterministicConstruction<'a> {
    pub fn new(instance: &'a Instance) -> Self {
        Self { instance }
    }

    /* 
    Simple placeholder construction heuristic: serve first gamma requests
    */
    fn construct_solution(&self) -> Solution<'a> {
        let n_reqs = self.instance.n_reqs();
        let gamma = self.instance.gamma();
        let n_vehicles = self.instance.n_vehicles();
        let capacity = self.instance.cap();
        let rho = self.instance.rho();

        let demands = self.instance.demands();
        let dist = self.instance.compute_distance_matrix();

        // Initialize each route starting and ending at the depot later
        let mut routes: Vec<Vec<usize>> = vec![vec![0]; n_vehicles];
        let mut loads = vec![0usize; n_vehicles];

        // Helper closures for node index lookup
        let pickup = |i: usize| 1 + i;
        let dropoff = |i: usize| 1 + n_reqs + i;

        // Small helper to compute route distance
        let route_distance = |route: &Vec<usize>| -> f64 {
            let mut d = 0.0;
            for w in route.windows(2) {
                d += dist[w[0]][w[1]] as f64;
            }
            d
        };

        // Compute Jain fairness for a vector of route distances
        let jain = |dists: &Vec<f64>| -> f64 {
            let sum: f64 = dists.iter().sum();
            let sum_sq: f64 = dists.iter().map(|x| x * x).sum();
            if sum_sq == 0.0 { return 1.0; }
            (sum * sum) / ((dists.len() as f64) * sum_sq)
        };

        for req_id in 0..gamma {
            let demand = demands[req_id];

            let mut best_vehicle = None;
            let mut best_score = f64::INFINITY;

            for k in 0..n_vehicles {
                if loads[k] + demand > capacity {
                    continue;
                }

                // Simulate appending pickup and dropoff
                let mut test_route = routes[k].clone();
                test_route.push(pickup(req_id));
                test_route.push(dropoff(req_id));

                // Compute new distances for fairness evaluation
                let mut dists: Vec<f64> = routes
                    .iter()
                    .map(|r| route_distance(r))
                    .collect();
                dists[k] = route_distance(&test_route);

                let delta_dist = dists[k] - route_distance(&routes[k]);
                let fairness = jain(&dists);

                let score = delta_dist + rho * (1.0 - fairness);

                if score < best_score {
                    best_score = score;
                    best_vehicle = Some(k);
                }
            }

            // If no feasible vehicle due to capacity, assign to least-loaded fallback
            let k = best_vehicle.unwrap_or_else(|| {
                loads.iter()
                    .enumerate()
                    .min_by_key(|(_, &load)| load)
                    .map(|(idx, _)| idx)
                    .unwrap()
            });

            routes[k].push(pickup(req_id));
            routes[k].push(dropoff(req_id));
            loads[k] += demand;
        }

        // End routes with depot
        for r in routes.iter_mut() {
            r.push(0);
        }

        Solution::new(self.instance, routes)
    }

}

impl<'a> Solver for DeterministicConstruction<'a> {
    fn solve(&self) -> Solution<'a> {
        self.construct_solution()
    }
}
