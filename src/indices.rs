use std::collections::*;

pub type Indices = HashSet<usize>;

pub trait SelectIndices {
    fn select_indices(self, indices: &Indices) -> Self;
    fn reject_indices(self, indices: &Indices) -> Self;
}

impl<T> SelectIndices for Vec<T> {
    fn select_indices(self, indices: &Indices) -> Self {
        self.into_iter()
            .enumerate()
            .filter(|&(i, _)| indices.contains(&i))
            .map(|(_, t)| t)
            .collect()
    }

    fn reject_indices(self, indices: &Indices) -> Self {
        self.into_iter()
            .enumerate()
            .filter(|&(i, _)| !indices.contains(&i))
            .map(|(_, t)| t)
            .collect()
    }
}
