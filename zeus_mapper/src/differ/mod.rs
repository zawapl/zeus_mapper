use std::fmt::Debug;

pub trait LogDifferences {
    fn log_differences(a: &Self, b: &Self, context: String);
}

impl<T> LogDifferences for Vec<T>
where
    T: LogDifferences,
{
    fn log_differences(a: &Self, b: &Self, context: String) {
        if a.len() != b.len() {
            println!("{context}: different length detected: {} vs {}", a.len(), b.len());
        }

        for i in 0..usize::min(a.len(), b.len()) {
            LogDifferences::log_differences(&a[i], &b[i], format!("{context}[{i:03}]"));
        }
    }
}

impl<const N: usize, T> LogDifferences for [T; N]
where
    T: LogDifferences,
{
    fn log_differences(a: &Self, b: &Self, context: String) {
        if a.len() != b.len() {
            println!("{context}: different length detected: {} vs {}", a.len(), b.len());
        }

        for i in 0..usize::min(a.len(), b.len()) {
            LogDifferences::log_differences(&a[i], &b[i], format!("{context}[{i:03}]"));
        }
    }
}

impl<A, B> LogDifferences for (A, B)
where
    A: LogDifferences + PartialEq + Debug,
    B: LogDifferences + PartialEq + Debug,
{
    fn log_differences(a: &Self, b: &Self, context: String) {
        if a != b {
            println!("{context}: difference: {a:?} vs {b:?}");
        }
    }
}

impl<T> LogDifferences for Option<T>
where
    T: PartialEq + Debug,
{
    fn log_differences(a: &Self, b: &Self, context: String) {
        if a != b {
            println!("{context}: difference: {a:?} vs {b:?}");
        }
    }
}

impl<T: DataConstant + PartialEq + Debug> LogDifferences for T {
    fn log_differences(a: &Self, b: &Self, context: String) {
        if a != b {
            println!("{context}: difference: {a:?} vs {b:?}");
        }
    }
}

macro_rules! default_differ_impl {
    ($x:tt) => {
        impl crate::prelude::LogDifferences for $x {
            fn log_differences(a: &Self, b: &Self, context: String) {
                if a != b {
                    println!("{context}: difference: {a:?} vs {b:?}");
                }
            }
        }
    };
}

use crate::prelude::DataConstant;
pub(crate) use default_differ_impl;

default_differ_impl!(bool);
default_differ_impl!(u8);
default_differ_impl!(u16);
default_differ_impl!(i16);
default_differ_impl!(u32);
default_differ_impl!(u64);
default_differ_impl!(usize);
default_differ_impl!(String);
