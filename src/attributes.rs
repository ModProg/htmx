//! Details on conversion for Attribute values.
use std::borrow::Cow;
use std::marker::PhantomData;
use std::num::{NonZeroU64, NonZeroU8};

use derive_more::Display;
use forr::forr;

/// An attribute that accepts an attribute value or a flag.
pub struct FlagOrValue<T>(PhantomData<T>);

/// An attribute that accepts any attribute value.
pub struct Any;

/// An attribute that accepts a numeric value.
pub struct Number;

/// An attribute that accepts a date and time.
pub struct DateTime;

/// An attribute that can be set as a flag or set to a value.
#[derive(Default, Debug, PartialEq, Eq, Hash)]
pub enum ValueOrFlag {
    /// Attribute is set to a value.
    Value(String),
    /// Attribute is set without a value.
    Flag,
    /// Attribute is not set.
    #[default]
    Unset,
}

impl ValueOrFlag {
    pub(crate) fn append<'a>(&mut self, value: impl Into<Cow<'a, str>>) {
        match self {
            ValueOrFlag::Value(c) => c.push_str(&value.into()),
            _ => *self = Self::Value(value.into().into()),
        }
    }
}

/// Converts to an Attribute that accepts type `Output`, e.g.,
/// [`Number`].
pub trait IntoAttribute<Output> {
    /// Converts into an attribute value.
    fn into_attribute(self) -> ValueOrFlag;
}

impl<A: IntoAttribute<T> + Clone, T> IntoAttribute<T> for &A {
    fn into_attribute(self) -> ValueOrFlag {
        self.clone().into_attribute()
    }
}

impl<A: IntoAttribute<T>, T> IntoAttribute<T> for Option<A> {
    fn into_attribute(self) -> ValueOrFlag {
        self.map_or(ValueOrFlag::Unset, IntoAttribute::into_attribute)
    }
}

macro_rules! into_attr {
    ($target:ident, $types:tt, |$self:ident| $($tmpl:tt)*) => {
        forr! { #type:ty in $types #*
            impl IntoAttribute<$target> for #type {
                fn into_attribute($self) -> ValueOrFlag {
                    $($tmpl)*
                }
            }
            impl IntoAttribute<Any> for #type {
                fn into_attribute($self) -> ValueOrFlag {
                    $($tmpl)*
                }
            }
            impl IntoAttribute<FlagOrValue<$target>> for #type {
                fn into_attribute($self) -> ValueOrFlag {
                    $($tmpl)*
                }
            }
        }
    }
}

into_attr! {
    Number,
    [u8, i8, u16, i16, u32, i32, f32, u64, i64, f64, u128, i128, isize, usize],
    |self| ValueOrFlag::Value(self.to_string())
}

into_attr! {
    String,
    [&str, String, Cow<'_, str>],
    |self| ValueOrFlag::Value(self.into())
}

into_attr! {  char, [char], |self| ValueOrFlag::Value(self.to_string()) }

// /// Trait accepted by an attribute that allows both values and flags.
// pub trait FlagOrAttributeValue {
//     /// Converts into value.
//     fn into_attribute(self) -> ValueOrFlag;
// }

impl IntoAttribute<bool> for bool {
    fn into_attribute(self) -> ValueOrFlag {
        if self {
            ValueOrFlag::Flag
        } else {
            ValueOrFlag::Unset
        }
    }
}

impl<T> IntoAttribute<FlagOrValue<T>> for bool {
    fn into_attribute(self) -> ValueOrFlag {
        IntoAttribute::<bool>::into_attribute(self)
    }
}

impl IntoAttribute<Any> for bool {
    fn into_attribute(self) -> ValueOrFlag {
        IntoAttribute::<bool>::into_attribute(self)
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

    impl<Tz: TimeZone> IntoAttribute<super::DateTime> for DateTime<Tz> {
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
