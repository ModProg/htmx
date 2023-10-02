use htmx::{htmx, HtmlPage};
use insta::assert_snapshot;

#[test]
fn html_page() {
    assert_snapshot!(
        htmx! {
            <HtmlPage mobile title="Title" lang="de" style_sheets=["hello.css", "world.css"]
                scripts=vec!["a_script.js".to_string()]>
            </_>
        }
        .to_string()
        .as_str()
    )
}
