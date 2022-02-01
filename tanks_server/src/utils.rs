use std::collections::HashSet;

use tanks_core::shared_types::Bullet;

pub trait VecOps<T, F: Fn(&mut T) -> bool> {
    fn drain_remove_if(&mut self, predicate: F) -> Vec<T>;
}

impl<T, F: Fn(&mut T) -> bool> VecOps<T, F> for Vec<T> {
    fn drain_remove_if(&mut self, predicate: F) -> Vec<T> {
        let mut return_vec = Vec::new();
        let mut i = 0;
        while i < self.len() {
            if predicate(&mut self[i]) {
                return_vec.push(self.remove(i));
            } else {
                i += 1;
            }
        }

        return_vec
    }
}

pub fn process_collisions(vec: &Vec<Bullet>) -> Vec<usize> {
    let mut set = HashSet::new();
    for i in 0..vec.len() {
        for j in 0..vec.len() {
            if i != j && vec[i].collides_with(&vec[j]) {
                set.insert(i);
                set.insert(j);
            }
        }
    }
    let mut vec = set.into_iter().collect::<Vec<usize>>();
    vec.sort();
    vec.reverse();

    vec
}
