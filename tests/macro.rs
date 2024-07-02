use htmx::native::a;
use htmx::{html, Fragment, Html, Tag};
use htmx_macros::component;

macro_rules! assert_html {
    ($html:tt$(, $rtml:tt)?) => {
        let html = html!$html;
        // $(assert_eq!(rtml!$rtml, html);)?
        insta::assert_snapshot!(html.into_string());
    };
}

#[allow(non_camel_case_types)]
struct custom<'html> {
    html: a<'html, Tag>,
}

impl<'html> custom<'html> {
    fn new(html: &'html mut Html) -> Self {
        Self { html: a::new(html) }
    }

    fn href(mut self, value: impl Into<String>) -> Self {
        self.html = self.html.href(value.into());
        self
    }

    fn close(self) -> impl htmx::IntoHtml {
        self.html.close();
        Fragment::EMPTY
    }
}

#[test]
fn test() {
    assert_html!((
        <div>
            <a href="hello" download/>
            <a href="hello" download="file.name"/>
            <custom href="test"/>
        </div>
    ), (
        div[
            a(href: "hello", download: true),
            a(href: "hello", download: "file.name"),
            Custom(href: "test")
        ]
    ));
}

#[test]
fn fn_component() {
    #[component]
    fn Component(a: bool, b: String) {
        html! {
            <button disabled=a>{b}</button>
        }
    }

    insta::assert_snapshot!(
        html! {
            <Component a b="Disabled Button"/>
            <Component a=true b="Disabled Button"/>
            <Component a=false b="Enabled Button"/>
            <Component b="Enabled Button"/>
        }
        .into_string()
    );
}

#[test]
fn reserved_attributes() {
    assert_html!({
        <script type_="module" />
        <script async_=true />
        // TODO <script type="module" />
        // <script async=true />
    }, {
        script(type_: "module"),
        script(async_: true),
        // TODO script(type: "module"),
        // script(async: true),
    });
}

#[test]
fn custom_element() {
    assert_html!({
        <custom-element attr="module">
            <p> "This is a child" </p>
        </_>
        <{"div"} custom_div="hello"> </_>
    });
}

#[test]
fn raw_html() {
    use htmx::RawSrc;
    assert_html!({
        "this < will be > escaped "
        <RawSrc("This < will > not")/>
    });
}

#[test]
fn controll_flow() {
    let mut b = [1, 2, 3].into_iter();
    let _b2 = b.clone();
    assert_html!({
        if true {
            <a>"Hello"</a>
        } else if false {
            <p>"else"</p>
        }
        for a in [1, 2, 3] {
            {format!("{a}")}
        }
        while let Some(b) = b.next() {
            {format!("{b}")}
        }
    }, {
        if true [
            a["Hello"]
        ] else if false [
            p["else"]
        ],
        for a in [1, 2, 3] [
            {format!("{a}")}
        ],
        while let Some(b) = b2.next() [
            {format!("{b}")}
        ]
    });
}
