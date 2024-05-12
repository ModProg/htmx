use chrono::{TimeZone, Utc};
use htmx::html;

#[test]
fn native() {
    insta::assert_snapshot!(
        html! {
            <del datetime=Utc.with_ymd_and_hms(2023, 10, 2, 21, 41, 36).unwrap()> "Deleted" </del>
            <object data="hello"/>
        }
        .to_string()
    );
}
