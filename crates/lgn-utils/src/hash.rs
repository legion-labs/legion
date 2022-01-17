use std::hash::{BuildHasher, Hash, Hasher};

use ahash::{AHasher, RandomState};
use siphasher::sip128::{Hasher128, SipHasher};

/// The `DefaultHash` trait is used to obtain a hash value for a single typed
/// value. It will rely on the default `Hasher` provided by the std library.
pub trait DefaultHash {
    fn default_hash(&self) -> u64;

    fn default_hash_128(&self) -> u128;
}

// Default implementation of DefaultHash for all types that implement the `Hash`
// trait.
impl<T> DefaultHash for T
where
    T: Hash,
{
    /// Returns the hash value for a single typed value, using `DefaultHasher`.
    fn default_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    /// Returns a 128-bit hash value for a single typed value, using
    /// `DefaultHasher128`.
    fn default_hash_128(&self) -> u128 {
        let mut hasher = DefaultHasher128::new();
        self.hash(&mut hasher);
        hasher.finish_128()
    }
}

pub struct DefaultHasher {}

impl DefaultHasher {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> AHasher {
        let builder = FixedState::default();
        builder.build_hasher()
    }
}

pub struct DefaultHasher128(SipHasher);

impl DefaultHasher128 {
    pub fn new() -> Self {
        Self(SipHasher::new())
    }

    pub fn finish_128(&self) -> u128 {
        self.0.finish128().as_u128()
    }
}

impl Default for DefaultHasher128 {
    fn default() -> Self {
        Self::new()
    }
}

impl Hasher for DefaultHasher128 {
    fn write(&mut self, bytes: &[u8]) {
        self.0.write(bytes);
    }

    fn finish(&self) -> u64 {
        self.0.finish()
    }
}

/// A hasher builder that will create a fixed hasher.
#[derive(Default)]
pub struct FixedState;

impl BuildHasher for FixedState {
    type Hasher = AHasher;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
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

/// A stable std hash map implementing `AHash`, a high speed keyed hashing
/// algorithm intended for use in in-memory hashmaps.
///
/// Unlike [`HashMap`] this has an iteration order that only depends on the
/// order of insertions and deletions and not a random source.
///
/// `AHash` is designed for performance and is NOT cryptographically secure.
pub type StableHashMap<K, V> = std::collections::HashMap<K, V, FixedState>;

/// A std hash set implementing `AHash`, a high speed keyed hashing algorithm
/// intended for use in in-memory hashmaps.
///
/// `AHash` is designed for performance and is NOT cryptographically secure.
pub type HashSet<K> = std::collections::HashSet<K, RandomState>;

/// A stable std hash set implementing `AHash`, a high speed keyed hashing
/// algorithm intended for use in in-memory hashmaps.
///
/// Unlike [`HashSet`] this has an iteration order that only depends on the
/// order of insertions and deletions and not a random source.
///
/// `AHash` is designed for performance and is NOT cryptographically secure.
pub type StableHashSet<K> = std::collections::HashSet<K, FixedState>;

pub trait AHashExt {
    fn new() -> Self
    where
        Self: Default + Sized,
    {
        Self::default()
    }

    fn with_capacity(capacity: usize) -> Self;
}

impl<K, V, S> AHashExt for std::collections::HashMap<K, V, S>
where
    S: Default,
{
    /// Creates an empty `HashMap` with the specified capacity with `AHash`.
    ///
    /// The hash map will be able to hold at least `capacity` elements without
    /// reallocating. If `capacity` is 0, the hash map will not allocate.
    ///
    /// # Examples
    ///
    /// ```
    /// use lgn_utils::{HashMap, AHashExt};
    /// let mut map: HashMap<&str, i32> = HashMap::with_capacity(10);
    /// assert!(map.capacity() >= 10);
    /// ```
    #[inline]
    fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, S::default())
    }
}

impl<K, S> AHashExt for std::collections::HashSet<K, S>
where
    S: Default,
{
    /// Creates an empty `HashSet` with the specified capacity with `AHash`.
    ///
    /// The hash set will be able to hold at least `capacity` elements without
    /// reallocating. If `capacity` is 0, the hash set will not allocate.
    ///
    /// # Examples
    ///
    /// ```
    /// use lgn_utils::{HashSet, AHashExt};
    /// let set: HashSet<i32> = HashSet::with_capacity(10);
    /// assert!(set.capacity() >= 10);
    /// ```
    #[inline]
    fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, S::default())
    }
}
