mod html;
mod special_components;

mod rusty;

pub use html::html;
pub use rusty::html as rtml;

// pub fn html(input: TokenStream) -> Result {
//     if input.is_empty() {
//         return Ok(quote!(::htmx::Html::new()));
//     }

//     let mut fork = input.clone().into_iter();

//     let first = fork.next();
//     let second = fork.next();

//     // TODO figure out actual differentiator
//     // probably would be, starts with `<` or starts with `{}` or `""` not
// followed     // by `,`

//     if matches!(input.peek(), Some(TokenTree::Punct(punct)) if
// punct.as_char() == '<') {         html::html(input.collect())
//     } else {
//         rusty::html(input.collect())
//     }
// }
