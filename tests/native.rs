use htmx::htmx;
use chrono::{Utc, TimeZone};

#[test]
fn native() {
    insta::assert_snapshot!(
        htmx! {
            <del datetime=Utc.with_ymd_and_hms(2023, 10, 2, 21, 41, 36).unwrap()> "Deleted" </del>
            <object data="hello"/>
        }
        .to_string()
    );
}
