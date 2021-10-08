pub use ahash::AHasher;
pub use instant::{Duration, Instant};
pub use uuid::Uuid;

pub use ahash::RandomState;

/// A hasher builder that will create a fixed hasher.
#[derive(Default)]
pub struct FixedState;

impl std::hash::BuildHasher for FixedState {
    type Hasher = AHasher;

    #[inline]
    fn build_hasher(&self) -> AHasher {
        AHasher::new_with_keys(
            0b1001010111101110000001001100010000000011001001101011001001111000,
            0b1100111101101011011110001011010100000100001111100011010011010101,
        )
    }
}

/// A std hash map implementing `AHash`, a high speed keyed hashing algorithm
/// intended for use in in-memory hashmaps.
///
/// `AHash` is designed for performance and is NOT cryptographically secure.
pub type HashMap<K, V> = std::collections::HashMap<K, V, RandomState>;

pub trait AHashExt {
    fn with_capacity(capacity: usize) -> Self;
}

impl<K, V> AHashExt for HashMap<K, V> {
    /// Creates an empty `HashMap` with the specified capacity with `AHash`.
    ///
    /// The hash map will be able to hold at least `capacity` elements without
    /// reallocating. If `capacity` is 0, the hash map will not allocate.
    ///
    /// # Examples
    ///
    /// ```
    /// use legion_utils::{HashMap, AHashExt};
    /// let mut map: HashMap<&str, i32> = HashMap::with_capacity(10);
    /// assert!(map.capacity() >= 10);
    /// ```
    #[inline]
    fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, RandomState::default())
    }
}

/// A stable std hash map implementing `AHash`, a high speed keyed hashing algorithm
/// intended for use in in-memory hashmaps.
///
/// Unlike [`HashMap`] this has an iteration order that only depends on the order
/// of insertions and deletions and not a random source.
///
/// `AHash` is designed for performance and is NOT cryptographically secure.
pub type StableHashMap<K, V> = std::collections::HashMap<K, V, FixedState>;

impl<K, V> AHashExt for StableHashMap<K, V> {
    /// Creates an empty `StableHashMap` with the specified capacity with `AHash`.
    ///
    /// The hash map will be able to hold at least `capacity` elements without
    /// reallocating. If `capacity` is 0, the hash map will not allocate.
    ///
    /// # Examples
    ///
    /// ```
    /// use legion_utils::{StableHashMap, AHashExt};
    /// let mut map: StableHashMap<&str, i32> = StableHashMap::with_capacity(10);
    /// assert!(map.capacity() >= 10);
    /// ```
    #[inline]
    fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, FixedState::default())
    }
}

/// A std hash set implementing `AHash`, a high speed keyed hashing algorithm
/// intended for use in in-memory hashmaps.
///
/// `AHash` is designed for performance and is NOT cryptographically secure.
pub type HashSet<K> = std::collections::HashSet<K, RandomState>;

impl<K> AHashExt for HashSet<K> {
    /// Creates an empty `HashSet` with the specified capacity with `AHash`.
    ///
    /// The hash set will be able to hold at least `capacity` elements without
    /// reallocating. If `capacity` is 0, the hash set will not allocate.
    ///
    /// # Examples
    ///
    /// ```
    /// use legion_utils::{HashSet, AHashExt};
    /// let set: HashSet<i32> = HashSet::with_capacity(10);
    /// assert!(set.capacity() >= 10);
    /// ```
    #[inline]
    fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, RandomState::default())
    }
}

/// A stable std hash set implementing `AHash`, a high speed keyed hashing algorithm
/// intended for use in in-memory hashmaps.
///
/// Unlike [`HashSet`] this has an iteration order that only depends on the order
/// of insertions and deletions and not a random source.
///
/// `AHash` is designed for performance and is NOT cryptographically secure.
pub type StableHashSet<K> = std::collections::HashSet<K, FixedState>;

impl<K> AHashExt for StableHashSet<K> {
    /// Creates an empty `StableHashSet` with the specified capacity with `AHash`.
    ///
    /// The hash set will be able to hold at least `capacity` elements without
    /// reallocating. If `capacity` is 0, the hash set will not allocate.
    ///
    /// # Examples
    ///
    /// ```
    /// use legion_utils::{StableHashSet, AHashExt};
    /// let set: StableHashSet<i32> = StableHashSet::with_capacity(10);
    /// assert!(set.capacity() >= 10);
    /// ```
    #[inline]
    fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, FixedState::default())
    }
}
