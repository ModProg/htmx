use htmx::{Children, Html, ToHtml};

#[derive(htmx::__private::typed_builder::TypedBuilder)]
#[builder(build_method(into = htmx::Html))]
struct Component {
    #[builder(via_mutators, mutators(
        fn child(&mut self, child: impl htmx::ToHtml) {
           self.children.push(child);
        }
    ))]
    children: Children,
}

impl From<Component> for Html {
    fn from(Component { children }: Component) -> Self {
        let mut html = Html::new();
        htmx::native::div::builder()
            .child(children)
            .build()
            .write_to_html(&mut html);
        html
    }
}

#[test]
fn test() {
    insta::assert_snapshot!(
        Component::builder()
            .child("hello")
            .child("world")
            .build()
            .to_string()
    )
}
