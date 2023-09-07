use htmx::htmx;
use htmx::{Html, ToHtml};

struct Custom {
    href: String,
}

impl Custom {
    fn builder() -> Self {
        Custom {
            href: "default".to_owned(),
        }
    }

    fn href(mut self, value: impl Into<String>) -> Self {
        self.href = value.into();
        self
    }

    fn build(self) -> Html {
        htmx! {
            <a href=self.href/>
        }
    }
}

#[test]
fn test() {
    let html = htmx!(
        <div>
            <a href="hello" download/>
            <a href="hello" download="file.name"/>
            <Custom href="test"/>
        </div>
    );
    insta::assert_snapshot!(html.into_html().to_string());
}
