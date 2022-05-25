use std::{collections::HashMap, hash::Hash};

/// A reference-counter.
///
/// Holds a positive reference count for each identifier.
#[derive(Debug, Clone)]
pub struct RefCounter<T> {
    refs: HashMap<T, usize>,
}

impl<T> Default for RefCounter<T> {
    fn default() -> Self {
        Self {
            refs: HashMap::default(),
        }
    }
}

impl<T: Eq + Hash + Clone> RefCounter<T> {
    /// Increment the reference count for the specified value.
    pub fn inc(&mut self, k: &T) {
        if let Some(v) = self.refs.get_mut(k) {
            *v += 1;
        } else {
            self.refs.insert(k.clone(), 1);
        }
    }

    /// Decrement the reference count for the specified value.
    ///
    /// # Returns
    ///
    /// If the value is referenced and still has a strictly positive reference
    /// count after the decrement, `false` is returned.
    ///
    /// If the value is no longer referenced, `true` is returned. This just
    /// happens the first time.
    pub fn dec(&mut self, k: &T) -> bool {
        if let Some(v) = self.refs.get_mut(k) {
            if *v > 1 {
                *v -= 1;
                false
            } else {
                self.refs.remove(k);
                true
            }
        } else {
            false
        }
    }

    /// Clear all references.
    pub fn clear(&mut self) {
        self.refs.clear();
    }

    /// Return all the values that have a strictly positive reference count.
    pub fn referenced(&self) -> Vec<&T> {
        self.refs
            .iter()
            .filter_map(|(k, v)| if *v > 0 { Some(k) } else { None })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;

    #[test]
    fn test_ref_counter() {
        let mut rc = RefCounter::default();

        rc.inc(&"apple");
        rc.inc(&"banana");
        rc.inc(&"cantaloupe");

        assert_eq!(
            rc.referenced().into_iter().sorted().collect::<Vec<_>>(),
            vec![&"apple", &"banana", &"cantaloupe"]
        );

        rc.inc(&"apple");
        assert!(!rc.dec(&"apple"));
        assert!(rc.dec(&"banana"));

        // A second time should not return `true`.
        assert!(!rc.dec(&"banana"));

        assert_eq!(
            rc.referenced().into_iter().sorted().collect::<Vec<_>>(),
            vec![&"apple", &"cantaloupe"]
        );

        rc.clear();

        assert_eq!(
            rc.referenced().into_iter().sorted().collect::<Vec<_>>(),
            Vec::<&&str>::new()
        );
    }
}
