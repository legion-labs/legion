#![allow(dead_code)]
use std::{borrow::Borrow, collections::BTreeMap, fmt::Debug, hash::Hash};

pub trait TreeVisitor<K, T> {
    fn visit_tree(&self, tree: &Tree<K, T>, depth: u8) {
        visit_tree(self, tree, depth);
    }

    fn visit_tree_key(&self, key: &K, depth: u8) {
        visit_tree_key(self, key, depth);
    }

    fn visit_value(&self, value: &T, depth: u8) {
        visit_value(self, value, depth);
    }
}

pub fn visit_tree<V, K, T>(visitor: &V, tree: &Tree<K, T>, depth: u8)
where
    V: TreeVisitor<K, T> + ?Sized,
{
    if let Some(ref value) = tree.value {
        visitor.visit_value(value, depth);
    }

    tree.sub.iter().for_each(|(key, sub_tree)| {
        visitor.visit_tree(sub_tree, depth);
        visitor.visit_tree_key(key, depth);
    });
}

pub fn visit_tree_key<V, K, T>(_visitor: &V, _key: &K, _depth: u8)
where
    V: TreeVisitor<K, T> + ?Sized,
{
}

pub fn visit_value<V, K, T>(_visitor: &V, _value: &T, _depth: u8)
where
    V: TreeVisitor<K, T> + ?Sized,
{
}

pub trait TreeVisitorMut<K, T> {
    fn visit_tree_mut(&mut self, tree: &Tree<K, T>, depth: u8) {
        visit_tree_mut(self, tree, depth);
    }

    fn visit_tree_key_mut(&mut self, key: &K, depth: u8) {
        visit_tree_key_mut(self, key, depth);
    }

    fn visit_value_mut(&mut self, value: &T, depth: u8) {
        visit_value_mut(self, value, depth);
    }
}

pub fn visit_tree_mut<V, K, T>(visitor: &mut V, tree: &Tree<K, T>, depth: u8)
where
    V: TreeVisitorMut<K, T> + ?Sized,
{
    if let Some(ref value) = tree.value {
        visitor.visit_value_mut(value, depth);
    }

    tree.sub.iter().for_each(|(key, sub_tree)| {
        visitor.visit_tree_mut(sub_tree, depth + 1);
        visitor.visit_tree_key_mut(key, depth + 1);
    });
}

pub fn visit_tree_key_mut<V, K, T>(_visitor: &mut V, _key: &K, _depth: u8)
where
    V: TreeVisitorMut<K, T> + ?Sized,
{
}

pub fn visit_value_mut<V, K, T>(_visitor: &mut V, _value: &T, _depth: u8)
where
    V: TreeVisitorMut<K, T> + ?Sized,
{
}

#[derive(Debug)]
pub struct Tree<K, T> {
    value: Option<T>,
    sub: BTreeMap<K, Tree<K, T>>,
}

impl<K, T> Default for Tree<K, T> {
    fn default() -> Self {
        Self {
            value: None,
            sub: BTreeMap::new(),
        }
    }
}

impl<K, T> Tree<K, T>
where
    K: Eq + Hash,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_value(value: T) -> Self {
        Self {
            value: Some(value),
            sub: BTreeMap::new(),
        }
    }

    pub fn value(&self) -> &Option<T> {
        &self.value
    }

    pub fn set_value(&mut self, value: T) {
        self.value = Some(value);
    }

    pub fn clear_value(&mut self) {
        self.value = None;
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&Self>
    where
        Q: Ord + ?Sized,
        K: Borrow<Q> + Ord,
    {
        self.sub.get(key)
    }

    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut Self>
    where
        Q: Ord + ?Sized,
        K: Borrow<Q> + Ord,
    {
        self.sub.get_mut(key)
    }

    pub fn get_or_else<F>(&mut self, key: K, f: F) -> Option<&Self>
    where
        F: Fn(&K) -> T,
        K: Clone + Ord,
    {
        self.insert_tree_or_else(key.clone(), f);

        self.get(&key)
    }

    pub fn get_or_default(&mut self, key: K) -> Option<&Self>
    where
        K: Clone + Ord,
    {
        self.insert_tree_or_default(key.clone());

        self.get(&key)
    }

    pub fn get_or_else_mut<F>(&mut self, key: K, f: F) -> Option<&mut Self>
    where
        F: Fn(&K) -> T,
        K: Clone + Ord,
    {
        self.insert_tree_or_else(key.clone(), f);

        self.get_mut(&key)
    }

    pub fn get_or_default_mut(&mut self, key: K) -> Option<&mut Self>
    where
        K: Clone + Ord,
    {
        self.insert_tree_or_default(key.clone());

        self.get_mut(&key)
    }

    pub fn get_at<Q>(&self, keys: &[&Q]) -> Option<&Self>
    where
        Q: Ord + ?Sized,
        K: Borrow<Q> + Ord,
    {
        let mut tree = self;

        for key in keys.iter() {
            if let Some(sub_tree) = tree.get(*key) {
                tree = sub_tree;
            } else {
                return None;
            }
        }

        Some(tree)
    }

    pub fn get_at_or_else<F>(&mut self, keys: Vec<K>, f: F) -> Option<&Self>
    where
        F: Fn(&K) -> T,
        K: Clone + Ord,
    {
        let mut tree = self;

        for key in keys.into_iter() {
            if let Some(sub_tree) = tree.get_or_else_mut(key, &f) {
                tree = sub_tree;
            } else {
                return None;
            }
        }

        Some(tree)
    }

    pub fn get_at_or_default(&mut self, keys: Vec<K>) -> Option<&Self>
    where
        K: Clone + Ord,
    {
        let mut tree = self;

        for key in keys.into_iter() {
            if let Some(sub_tree) = tree.get_or_default_mut(key) {
                tree = sub_tree;
            } else {
                return None;
            }
        }

        Some(tree)
    }

    pub fn get_at_mut<Q>(&mut self, keys: &[&Q]) -> Option<&mut Self>
    where
        Q: Ord + ?Sized,
        K: Borrow<Q> + Ord,
    {
        let mut tree = self;

        for key in keys.iter() {
            if let Some(sub_tree) = tree.get_mut(key) {
                tree = sub_tree;
            } else {
                return None;
            }
        }

        Some(tree)
    }

    pub fn get_at_or_else_mut<F>(&mut self, keys: Vec<K>, f: F) -> Option<&mut Self>
    where
        F: Fn(&K) -> T,
        K: Clone + Ord,
    {
        let mut tree = self;

        for key in keys.into_iter() {
            if let Some(sub_tree) = tree.get_or_else_mut(key, &f) {
                tree = sub_tree;
            } else {
                return None;
            }
        }

        Some(tree)
    }

    pub fn get_at_or_default_mut(&mut self, keys: Vec<K>) -> Option<&mut Self>
    where
        K: Clone + Ord,
    {
        let mut tree = self;

        for key in keys.into_iter() {
            if let Some(sub_tree) = tree.get_or_default_mut(key) {
                tree = sub_tree;
            } else {
                return None;
            }
        }

        Some(tree)
    }

    pub fn insert(&mut self, key: K, value: Option<T>) -> bool
    where
        K: Ord,
    {
        let sub_tree = match value {
            Some(value) => Self::from_value(value),
            None => Self::new(),
        };

        self.insert_tree(key, sub_tree)
    }

    fn insert_tree_or_else<F>(&mut self, key: K, f: F) -> bool
    where
        F: Fn(&K) -> T,
        K: Clone + Ord,
    {
        let value = f(&key);

        self.insert_tree(key, Self::from_value(value))
    }

    fn insert_tree_or_default(&mut self, key: K) -> bool
    where
        K: Ord,
    {
        self.insert_tree(key, Self::default())
    }

    fn insert_tree(&mut self, key: K, sub_tree: Tree<K, T>) -> bool
    where
        K: Ord,
    {
        if self.sub.contains_key(&key) {
            return false;
        }

        self.sub.insert(key, sub_tree).is_none()
    }

    pub fn insert_at(&mut self, keys: Vec<K>, value: T) -> bool
    where
        K: Clone + Ord,
    {
        if let Some(tree) = self.get_at_or_default_mut(keys) {
            tree.set_value(value);

            return true;
        }

        false
    }

    pub fn insert_at_or_else<F>(&mut self, keys: Vec<K>, value: T, f: F) -> bool
    where
        F: Fn(&K) -> T,
        K: Clone + Ord,
    {
        if let Some(tree) = self.get_at_or_else_mut(keys, &f) {
            tree.set_value(value);

            return true;
        }

        false
    }
}
