use statistical::{mean, median, mode};
use std::hash::Hash;
use num_traits::float::Float;

#[derive(Debug)]
pub struct Stats<T, G> {
    pub max: Option<T>,
    pub min: Option<T>,
    pub mean: G,
    pub median: G,
    pub mode: Option<T>,
}

impl<T, G> Stats<T, G>
where
{
    pub fn resolve<F>(self, f: F) -> Stats<T, T>
    where
        F: Fn(G) -> T
    {
        Stats {
            max: self.max,
            min: self.min,
            mean: f(self.mean),
            median: f(self.median),
            mode: self.mode,
        }
    }
}

impl<T, G> Stats<T, G>
where
    T: Ord + Hash + Copy,
    G: Float
{
    pub fn new<F>(xs: &[T], f: F) -> Stats<T, G>
    where
        F: FnMut(&T) -> G
    {
        let max = xs.iter().max();
        let min = xs.iter().min();
        let mode = mode(xs);

        let ys: Vec<G> = xs.iter().map(f).collect();
        let mean = mean(&ys);
        let median = median(&ys);

        Stats {
            max: max.cloned(),
            min: min.cloned(),
            mean,
            median,
            mode,
        }
    }
}

impl<T: std::fmt::Debug, G: std::fmt::Debug> Stats<T, G>
{
    pub fn print_stats(&self) -> String {
        format!(
            "Max: {:?}\n\tMin: {:?}\n\tMean: {:?}\n\tMedian: {:?}\n\tMode: {:?}",
            self.max,
            self.min,
            self.mean,
            self.median,
            self.mode
        )
    }
}
