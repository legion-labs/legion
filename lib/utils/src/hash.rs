use ahash::{AHasher, RandomState};
use std::hash::Hasher;

/// The `DefaultHash` trait is used to obtain a hash value for a single typed value.
/// It will rely on the default `Hasher` provided by the std library.
pub trait DefaultHash {
    fn default_hash(&self) -> u64;
}

// Default implementation of DefaultHash for all types that implement the `Hash` trait.
impl<T> DefaultHash for T
where
    T: std::hash::Hash,
{
    /// Returns the hash value for a single typed value, using the default `Hasher` from `HashMap`.
    fn default_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

pub type DefaultHasher = FixedState;

/// A hasher builder that will create a fixed hasher.
pub struct FixedState(AHasher);

impl FixedState {
    pub fn new() -> Self {
        Self(AHasher::new_with_keys(
            0b1001010111101110000001001100010000000011001001101011001001111000,
            0b1100111101101011011110001011010100000100001111100011010011010101,
        ))
    }
}

impl Default for FixedState {
    fn default() -> Self {
        Self::new()
    }
}

impl std::hash::BuildHasher for FixedState {
    type Hasher = Self;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        Self::new()
    }
}

impl Hasher for FixedState {
    #[inline]
    fn write(&mut self, msg: &[u8]) {
        self.0.write(msg);
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.0.finish()
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
