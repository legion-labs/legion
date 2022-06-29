use std::{sync::Arc, time::Duration};

use downcast_rs::{impl_downcast, Downcast};
use lgn_math::{Mat3, Mat4, Quat, Vec2, Vec3, Vec4};
use uuid::Uuid;

pub trait AnyEq: Downcast {
    // Perform the test.
    fn equals_a(&self, _: &dyn AnyEq) -> bool;
}
impl_downcast!(AnyEq);

impl PartialEq for dyn AnyEq + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.equals_a(other)
    }
}

// Implement A for all 'static types implementing PartialEq.
impl<T: 'static + PartialEq> AnyEq for T {
    fn equals_a(&self, other: &dyn AnyEq) -> bool {
        // Do a type-safe casting. If the types are different,
        // return false, otherwise test the values for equality.
        other.downcast_ref::<T>().map_or(false, |a| self == a)
    }
}

#[derive(Clone)]
pub enum Type {
    Signal,
    Bool(bool),
    Int(i64),
    Float(f64),
    Vec2(Vec2),
    Vec3(Vec3),
    Vec4(Vec4),
    Quat(Quat),
    Mat3(Mat3),
    Mat4(Mat4),
    String(String),
    Uuid(Uuid),
    Duration(Duration),
    Dynamic(Arc<Box<dyn AnyEq>>),
    Vector(Vec<Type>),
}

impl Default for Type {
    fn default() -> Self {
        Type::Signal
    }
}

impl TryFrom<Type> for String {
    type Error = &'static str;

    fn try_from(value: Type) -> Result<Self, Self::Error> {
        match value {
            Type::String(val) => Ok(val),
            Type::Bool(val) => Ok(val.to_string()),
            Type::Int(val) => Ok(val.to_string()),
            Type::Float(val) => Ok(val.to_string()),
            Type::Vec2(val) => Ok(val.to_string()),
            Type::Vec3(val) => Ok(val.to_string()),
            Type::Vec4(val) => Ok(val.to_string()),
            Type::Quat(val) => Ok(val.to_string()),
            Type::Mat3(val) => Ok(val.to_string()),
            Type::Mat4(val) => Ok(val.to_string()),
            Type::Uuid(val) => Ok(val.to_string()),
            _ => Err("Cannot convert to String"),
        }
    }
}

impl From<String> for Type {
    fn from(value: String) -> Self {
        Type::String(value)
    }
}

impl TryFrom<Type> for i64 {
    type Error = String;

    fn try_from(value: Type) -> Result<Self, Self::Error> {
        match value {
            Type::Int(val) => Ok(val),
            Type::Float(val) => Ok(val as i64),
            Type::String(val) => match val.parse::<i64>() {
                Ok(parsed_val) => Ok(parsed_val),
                Err(err) => Err(format!("Cannot convert {} to i64: {}", val, err)),
            },
            _ => Err("Cannot convert to i64".to_string()),
        }
    }
}

impl From<i64> for Type {
    fn from(value: i64) -> Self {
        Type::Int(value)
    }
}

impl TryFrom<Type> for f64 {
    type Error = String;

    fn try_from(value: Type) -> Result<Self, Self::Error> {
        match value {
            #[allow(clippy::cast_precision_loss)]
            Type::Int(val) => Ok(val as f64),
            Type::Float(val) => Ok(val),
            Type::String(val) => match val.parse::<f64>() {
                Ok(parsed_val) => Ok(parsed_val),
                Err(err) => Err(format!("Cannot convert {} to f64: {}", val, err)),
            },
            _ => Err("Cannot convert to f64".to_string()),
        }
    }
}

impl From<f64> for Type {
    fn from(value: f64) -> Self {
        Type::Float(value)
    }
}
