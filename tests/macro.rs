use htmx::{component, html, native::a, rtml, Html, Tag, WriteHtml};

macro_rules! assert_html {
    ($html:tt$(, $rtml:tt)?) => {
        let html = html!$html;
        $(assert_eq!(rtml!$rtml, html);)?
        insta::assert_snapshot!(html.to_string());
    };
}

struct Custom<Html: WriteHtml> {
    html: a<Html, Tag>,
}

impl<Html: WriteHtml> Custom<Html> {
    fn new(html: Html) -> Self {
        Self { html: a::new(html) }
    }

    fn href(mut self, value: impl Into<String>) -> Self {
        self.html.href(value.into());
        self
    }
}

impl<Html: WriteHtml> Drop for Custom<Html> {
    fn drop(&mut self) {
        todo!()
    }
}

#[test]
fn test() {
    assert_html!((
        <div>
            <a href="hello" download/>
            <a href="hello" download="file.name"/>
            <Custom href="test"/>
        </div>
    ), (
        div[
            a(href: "hello", download: true),
            a(href: "hello", download: "file.name"),
            Custom(href: "test")
        ]
    ));
}

// TODO
// #[test]
// fn struct_component() {
//     #[component]
//     struct Component {
//         a: bool,
//         b: String,
//     }

//     impl From<Component> for Html {
//         fn from(Component { a, b }: Component) -> Self {
//             html! {
//                 <button disabled=a>{b}</button>
//             }
//         }
//     }
//     assert_html!({
//             <Component a b="Disabled Button"/>
//             <Component a=true b="Disabled Button"/>
//             <Component a=false b="Enabled Button"/>
//             <Component b="Enabled Button"/>
//     }, [
//        Component(a: true, b: "Disabled Button"),
//        Component(a: true, b: "Disabled Button"),
//        Component(a: false, b: "Enabled Button"),
//        Component(b: "Enabled Button"),
//     ]);
// }

// #[test]
// fn fn_component() {
//     #[component]
//     fn Component(a: bool, b: String) -> Html {
//         html! {
//             <button disabled=a>{b}</button>
//         }
//     }

//     insta::assert_snapshot!(
//         html! {
//             <Component a b="Disabled Button"/>
//             <Component a=true b="Disabled Button"/>
//             <Component a=false b="Enabled Button"/>
//             <Component b="Enabled Button"/>
//         }
//         .to_string()
//     );
// }

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
    use htmx::RawHtml;
    assert_html!({
        "this < will be > escaped "
        <RawHtml("This < will > not")/>
    });
}

#[test]
fn controll_flow() {
    let mut b = [1, 2, 3].into_iter();
    let mut b2 = b.clone();
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
