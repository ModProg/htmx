use htmx::{htmx, Html};
use htmx_macros::component;

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
    insta::assert_snapshot!(html.to_string());
}

#[test]
fn struct_componentr() {
    #[component]
    struct Component {
        a: bool,
        b: String,
    }

    impl From<Component> for Html {
        fn from(Component { a, b }: Component) -> Self {
            htmx! {
                <button disabled=a>{b}</button>
            }
        }
    }

    insta::assert_snapshot!(
        htmx! {
            <Component a b="Disabled Button"/>
            <Component a=true b="Disabled Button"/>
            <Component a=false b="Enabled Button"/>
            <Component b="Enabled Button"/>
        }
        .to_string()
    );
}

#[test]
fn fn_component() {
    #[component]
    fn Component(a: bool, b: String) -> Html {
        htmx! {
            <button disabled=a>{b}</button>
        }
    }

    insta::assert_snapshot!(
        htmx! {
            <Component a b="Disabled Button"/>
            <Component a=true b="Disabled Button"/>
            <Component a=false b="Enabled Button"/>
            <Component b="Enabled Button"/>
        }
        .to_string()
    );
}
