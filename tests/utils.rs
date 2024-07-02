use htmx::{html, HtmlPage};
use insta::assert_snapshot;

#[test]
fn html_page() {
    assert_snapshot!(
        html! {
            <HtmlPage mobile title="Title" lang="de" style_sheets=["hello.css", "world.css"]
                scripts=vec!["a_script.js".to_string()] >
                <a> </a>
            </_>
        }
        .into_string()
        .as_str()
    )
}
