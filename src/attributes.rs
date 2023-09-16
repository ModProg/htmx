//! Details on conversion for Attribute values.
use std::borrow::Cow;

use forr::forr;

/// An attribute that accepts a numeric value.
pub struct Number;

/// An attribute that can be just set or set to a value.
pub enum ValueOrFlag {
    /// Attribute is set to a value.
    Value(String),
    /// Attribute is set without a value.
    Flag,
    /// Attribute is not set.
    Unset,
}

/// Converts to an Attribute that accepts type [`Self::Target`], e.g.,
/// [`Number`].
pub trait IntoAttribute {
    /// Target value of the attribute.
    type Target;
    /// Converts into an attribute value.
    fn into_attribute(self) -> ValueOrFlag;
}

impl<A: IntoAttribute + Clone> IntoAttribute for &A {
    type Target = A::Target;

    fn into_attribute(self) -> ValueOrFlag {
        self.clone().into_attribute()
    }
}

impl<A: IntoAttribute> IntoAttribute for Option<A> {
    type Target = A::Target;

    fn into_attribute(self) -> ValueOrFlag {
        self.map_or(ValueOrFlag::Unset, IntoAttribute::into_attribute)
    }
}

impl IntoAttribute for bool {
    type Target = bool;

    fn into_attribute(self) -> ValueOrFlag {
        if self {
            ValueOrFlag::Flag
        } else {
            ValueOrFlag::Unset
        }
    }
}

impl IntoAttribute for char {
    type Target = char;

    fn into_attribute(self) -> ValueOrFlag {
        ValueOrFlag::Value(self.to_string())
    }
}

forr! { $type:ty in [u8, i8, u16, i16, u32, i32, f32, u64, i64, f64, u128, i128, isize, usize] $*
    impl IntoAttribute for $type {
        type Target = Number;
        fn into_attribute(self) -> ValueOrFlag {
            ValueOrFlag::Value(self.to_string())
        }
    }
}

forr! { $type:ty in [&str, String, Cow<'_, str>] $*
    impl IntoAttribute for $type {
        type Target = String;
        fn into_attribute(self) -> ValueOrFlag {
            ValueOrFlag::Value(self.into())
        }
    }
}

/// Trait accepted by an attribute that allows both values and flags.
pub trait FlagOrAttributeValue {
    /// Converts into value.
    fn into_attribute(self) -> ValueOrFlag;
}

impl FlagOrAttributeValue for bool {
    fn into_attribute(self) -> ValueOrFlag {
        IntoAttribute::into_attribute(self)
    }
}

impl<A: IntoAttribute<Target = String>> FlagOrAttributeValue for A {
    fn into_attribute(self) -> ValueOrFlag {
        self.into_attribute()
    }
}

/// Trait accepted by an attribute that accepts any value
pub trait AnyAttributeValue {
    /// Converts into value.
    fn into_attribute(self) -> ValueOrFlag;
}

impl<T: IntoAttribute> AnyAttributeValue for T {
    fn into_attribute(self) -> ValueOrFlag {
        self.into_attribute()
    }
}
