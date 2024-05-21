//! Details on conversion for Attribute values.
use std::borrow::Cow;
use std::marker::PhantomData;
use std::num::{NonZeroU64, NonZeroU8};

use derive_more::Display;
use forr::forr;

use crate::WriteHtml;

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

/// Converts to an Attribute that accepts type `Output`, e.g.,
/// [`Number`].
pub trait ToAttribute<Output> {
    /// Converts into an attribute value.
    fn write(&self, html: impl WriteHtml);
    fn write_inner(&self, html: impl WriteHtml);
    fn is_unset(&self) -> bool {
        false
    }
}

impl<A: ToAttribute<T>, T> ToAttribute<T> for &A {
    fn write(&self, html: impl WriteHtml) {
        <A as ToAttribute<T>>::write(self, html);
    }

    fn write_inner(&self, html: impl WriteHtml) {
        <A as ToAttribute<T>>::write_inner(self, html);
    }

    fn is_unset(&self) -> bool {
        <A as ToAttribute<T>>::is_unset(self)
    }
}

impl<A: ToAttribute<T>, T> ToAttribute<T> for Option<A> {
    fn write(&self, html: impl WriteHtml) {
        self.as_ref().unwrap().write(html);
    }

    fn write_inner(&self, html: impl WriteHtml) {
        self.as_ref().unwrap().write(html);
    }

    fn is_unset(&self) -> bool {
        self.is_none()
    }
}

macro_rules! into_attr {
    ($target:ident, $types:tt, $fn:ident, $fn_inner:ident) => {
        forr! { #type:ty in $types #*
            forr! {#gen:ty in [$target, Any, FlagOrValue<$target>] #*
                impl ToAttribute<#gen> for #type {
                    fn write(&self, mut html: impl WriteHtml) {
                        html.$fn(self)
                    }
                    fn write_inner(&self, mut html: impl WriteHtml) {
                        html.$fn_inner(self)
                    }
                }
            }
        }
    };
}

into_attr! {
    Number,
    [u8, i8, u16, i16, u32, i32, f32, u64, i64, f64, u128, i128, isize, usize],
    write_attr_value_unchecked,
    write_attr_value_inner_unchecked
}

into_attr! {
    String,
    [&str, String, Cow<'_, str>],
    write_attr_value_encoded,
    write_attr_value_inner_encoded
}

into_attr! {  char, [char], write_attr_value_encoded, write_attr_value_inner_encoded }

// /// Trait accepted by an attribute that allows both values and flags.
// pub trait FlagOrAttributeValue {
//     /// Converts into value.
//     fn into_attribute(self) -> ValueOrFlag;
// }

impl ToAttribute<bool> for bool {
    fn write(&self, _html: impl WriteHtml) {}

    fn write_inner(&self, _html: impl WriteHtml) {}

    fn is_unset(&self) -> bool {
        !*self
    }
}

impl<T> ToAttribute<FlagOrValue<T>> for bool {
    fn write(&self, _html: impl WriteHtml) {}

    fn write_inner(&self, _html: impl WriteHtml) {}

    fn is_unset(&self) -> bool {
        !*self
    }
}

impl ToAttribute<Any> for bool {
    fn write(&self, _html: impl WriteHtml) {}

    fn write_inner(&self, _html: impl WriteHtml) {}

    fn is_unset(&self) -> bool {
        !*self
    }
}

/// An attribute that accepts the date time according to [`<time>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/time#valid_datetime_values).
///
/// The most important implementers are the [`chrono`](::chrono) types as well
/// as the tuples for [`Year`], [`Week`] and [`Day`].
pub trait TimeDateTime {
    /// Converts into value.
    fn write(&self, html: impl WriteHtml);

    fn is_unset(&self) -> bool {
        false
    }
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

    use super::{Day, TimeDateTime, ToAttribute, Week, WriteHtml, Year};

    impl<Tz: TimeZone> ToAttribute<super::DateTime> for DateTime<Tz> {
        fn write(&self, mut html: impl WriteHtml) {
            html.write_attr_value_unchecked(self.to_rfc3339());
        }

        fn write_inner(&self, mut html: impl WriteHtml) {
            html.write_attr_value_inner_unchecked(self.to_rfc3339());
        }
    }

    impl TimeDateTime for (Year, Month) {
        fn write(&self, mut html: impl WriteHtml) {
            html.write_attr_value_unchecked(format_args!(
                "{}-{:02}",
                self.0,
                self.1.number_from_month()
            ));
        }
    }

    impl TimeDateTime for NaiveDate {
        fn write(&self, mut html: impl WriteHtml) {
            html.write_attr_value_unchecked(self.format("%Y-%m-%d"));
        }
    }

    impl TimeDateTime for (Month, Day) {
        fn write(&self, mut html: impl WriteHtml) {
            html.write_attr_value_unchecked(format_args!(
                "{:02}-{}",
                self.0.number_from_month(),
                self.1
            ));
        }
    }

    impl TimeDateTime for NaiveTime {
        fn write(&self, mut html: impl WriteHtml) {
            html.write_attr_value_unchecked(self.format("%H:%M:%S.3f").to_string());
        }
    }
    impl TimeDateTime for NaiveDateTime {
        fn write(&self, mut html: impl WriteHtml) {
            html.write_attr_value_unchecked(self.format("%Y-%m-%d %H:%M:%S.3f").to_string());
        }
    }

    impl TimeDateTime for Utc {
        fn write(&self, mut html: impl WriteHtml) {
            html.write_attr_value_unchecked("Z");
        }
    }

    impl TimeDateTime for Local {
        fn write(&self, mut html: impl WriteHtml) {
            html.write_attr_value_unchecked(Self::now().format("%z"));
        }
    }
    impl TimeDateTime for FixedOffset {
        fn write(&self, mut html: impl WriteHtml) {
            html.write_attr_value_unchecked(self.to_string());
        }
    }

    impl<Tz: TimeZone> TimeDateTime for DateTime<Tz> {
        fn write(&self, mut html: impl WriteHtml) {
            html.write_attr_value_unchecked(self.to_rfc3339());
        }
    }

    impl TimeDateTime for (Year, Week) {
        fn write(&self, mut html: impl WriteHtml) {
            html.write_attr_value_unchecked(format_args!("{}-{}", self.0, self.1));
        }
    }

    impl TimeDateTime for Year {
        fn write(&self, mut html: impl WriteHtml) {
            html.write_attr_value_unchecked(format_args!("{:04}", self.0));
        }
    }

    impl TimeDateTime for Duration {
        fn write(&self, mut html: impl WriteHtml) {
            html.write_attr_value_unchecked(self);
        }
    }
}
