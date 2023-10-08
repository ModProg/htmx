//! Details on conversion for Attribute values.
use std::borrow::Cow;
use std::num::{NonZeroU64, NonZeroU8};

use derive_more::Display;
use forr::forr;

/// An attribute that accepts a numeric value.
pub struct Number;

/// An attribute that accepts a date and time.
pub struct DateTime;

/// An attribute that can be set as a flag or set to a value.
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

/// An attribute that accepts the datetime according to [`<time>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/time#valid_datetime_values).
///
/// The most important implementers are the [`chrono`](::chrono) types as well
/// as the tuples for [`Year`], [`Week`] and [`Day`].
pub trait TimeDateTime {
    /// Converts into value.
    fn into_attribute(self) -> ValueOrFlag;
}

/// Year in a `<time datetime={}>`.
#[derive(Display)]
pub struct Year(#[display("{:04}")] pub NonZeroU64);
impl Year {
    /// Returns [`None`] if `year` is `0`.
    pub fn new(year: u64) -> Option<Self> {
        NonZeroU64::new(year).map(Self)
    }
}

/// Day in a `<time datetime={}>`.
#[derive(Display)]
pub struct Day(#[display("{:02}")] NonZeroU8);
impl Day {
    /// Returns [`None`] if `day == 0` or `day > 31`.
    pub fn new(day: u8) -> Option<Self> {
        if day > 31 {
            return None;
        }
        NonZeroU8::new(day).map(Self)
    }
}

/// Week in a `<time datetime={}>`.
#[derive(Display)]
pub struct Week(#[display("W{:02}")] NonZeroU8);
impl Week {
    /// Returns [`None`] if `week == 0` or `week > 53`.
    pub fn new(week: u8) -> Option<Self> {
        if week > 53 {
            return None;
        }
        NonZeroU8::new(week).map(Self)
    }
}

mod chrono {
    use chrono::{
        DateTime, Duration, FixedOffset, Local, Month, NaiveDate, NaiveDateTime, NaiveTime,
        TimeZone, Utc,
    };

    use super::{Day, IntoAttribute, TimeDateTime, ValueOrFlag, Week, Year};

    impl<Tz: TimeZone> IntoAttribute for DateTime<Tz> {
        type Target = super::DateTime;

        fn into_attribute(self) -> ValueOrFlag {
            ValueOrFlag::Value(self.to_rfc3339())
        }
    }

    impl TimeDateTime for (Year, Month) {
        fn into_attribute(self) -> ValueOrFlag {
            ValueOrFlag::Value(format!("{}-{:02}", self.0, self.1.number_from_month()))
        }
    }

    impl TimeDateTime for NaiveDate {
        fn into_attribute(self) -> ValueOrFlag {
            ValueOrFlag::Value(self.format("%Y-%m-%d").to_string())
        }
    }

    impl TimeDateTime for (Month, Day) {
        fn into_attribute(self) -> ValueOrFlag {
            ValueOrFlag::Value(format!("{:02}-{}", self.0.number_from_month(), self.1))
        }
    }

    impl TimeDateTime for NaiveTime {
        fn into_attribute(self) -> ValueOrFlag {
            ValueOrFlag::Value(self.format("%H:%M:%S.3f").to_string())
        }
    }
    impl TimeDateTime for NaiveDateTime {
        fn into_attribute(self) -> ValueOrFlag {
            ValueOrFlag::Value(self.format("%Y-%m-%d %H:%M:%S.3f").to_string())
        }
    }

    impl TimeDateTime for Utc {
        fn into_attribute(self) -> ValueOrFlag {
            ValueOrFlag::Value("Z".into())
        }
    }

    impl TimeDateTime for Local {
        fn into_attribute(self) -> ValueOrFlag {
            ValueOrFlag::Value(Self::now().format("%z").to_string())
        }
    }
    impl TimeDateTime for FixedOffset {
        fn into_attribute(self) -> ValueOrFlag {
            ValueOrFlag::Value(self.to_string())
        }
    }

    impl<Tz: TimeZone> TimeDateTime for DateTime<Tz> {
        fn into_attribute(self) -> ValueOrFlag {
            ValueOrFlag::Value(self.to_rfc3339())
        }
    }

    impl TimeDateTime for (Year, Week) {
        fn into_attribute(self) -> ValueOrFlag {
            ValueOrFlag::Value(format!("{}-{}", self.0, self.1))
        }
    }

    impl TimeDateTime for Year {
        fn into_attribute(self) -> ValueOrFlag {
            ValueOrFlag::Value(format!("{:04}", self.0))
        }
    }

    impl TimeDateTime for Duration {
        fn into_attribute(self) -> ValueOrFlag {
            ValueOrFlag::Value(self.to_string())
        }
    }
}
