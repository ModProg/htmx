/// ```no_test
/// T![[]];
/// ```
macro_rules! T {
    [[]] => {
        syn::token::Bracket
    };
    [{}] => {
        syn::token::Brace
    };
    [()] => {
        syn::token::Paren
    };
    [$tt:tt] => {
        syn::Token! {$tt}
    };
}

// macro_rules! custom_keyword {
//     ($ident:ident as $Struct:ident) => {
//         #[allow(non_camel_case_types)]
//         pub struct $Struct {
//             pub span: syn::__private::Span,
//         }
//
//         #[doc(hidden)]
//         #[allow(dead_code, non_snake_case)]
//         pub fn $ident<__S:
// syn::__private::IntoSpans<syn::__private::Span>>(span: __S) -> $Struct {
//             $Struct {
//                 span: syn::__private::IntoSpans::into_spans(span),
//             }
//         }
//
//         const _: () = {
//             impl syn::__private::Default for $Struct {
//                 fn default() -> Self {
//                     $Struct {
//                         span: syn::__private::Span::call_site(),
//                     }
//                 }
//             }
//
//             impl_parse_for_custom_keyword!($ident as $Struct);
//             impl_to_tokens_for_custom_keyword!($ident as $Struct);
//             // impl_clone_for_custom_keyword!($ident);
//             // impl_extra_traits_for_custom_keyword!($ident);
//         };
//     };
// }
//
// macro_rules! impl_parse_for_custom_keyword {
//     ($ident:ident as $Struct:ident) => {
//         // For peek.
//         impl syn::token::CustomToken for $Struct {
//             fn peek(cursor: syn::buffer::Cursor) -> syn::__private::bool {
//                 if let syn::__private::Some((ident, _rest)) = cursor.ident()
// {                     ident == syn::__private::stringify!($ident)
//                 } else {
//                     false
//                 }
//             }
//
//             fn display() -> &'static syn::__private::str {
//                 syn::__private::concat!("`",
// syn::__private::stringify!($ident), "`")             }
//         }
//
//         impl syn::parse::Parse for $Struct {
//             fn parse(input: syn::parse::ParseStream) ->
// syn::parse::Result<$Struct> {                 input.step(|cursor| {
//                     if let syn::__private::Some((ident, rest)) =
// cursor.ident() {                         if ident ==
// syn::__private::stringify!($ident) {                             return
// syn::__private::Ok(($Struct { span: ident.span() }, rest));
// }                     }
//                     syn::__private::Err(cursor.error(syn::__private::concat!(
//                         "expected `",
//                         syn::__private::stringify!($ident),
//                         "`",
//                     )))
//                 })
//             }
//         }
//     };
// }
//
// #[macro_export]
// macro_rules! impl_to_tokens_for_custom_keyword {
//     ($ident:ident as $Struct:ident) => {
//         impl syn::__private::ToTokens for $Struct {
//             fn to_tokens(&self, tokens: &mut syn::__private::TokenStream2) {
//                 let ident =
// syn::Ident::new(syn::__private::stringify!($ident), self.span);
// syn::__private::TokenStreamExt::append(tokens, ident);             }
//         }
//     };
// }
//
// // Not public API.
// #[cfg(feature = "clone-impls")]
// #[doc(hidden)]
// #[macro_export]
// macro_rules! impl_clone_for_custom_keyword {
//     ($ident:ident) => {
//         impl syn::__private::Copy for $ident {}
//
//         #[allow(clippy::expl_impl_clone_on_copy)]
//         impl syn::__private::Clone for $ident {
//             fn clone(&self) -> Self {
//                 *self
//             }
//         }
//     };
// }
//
// // Not public API.
// #[cfg(not(feature = "clone-impls"))]
// #[doc(hidden)]
// #[macro_export]
// macro_rules! impl_clone_for_custom_keyword {
//     ($ident:ident) => {};
// }
//
// // Not public API.
// #[cfg(feature = "extra-traits")]
// #[doc(hidden)]
// #[macro_export]
// macro_rules! impl_extra_traits_for_custom_keyword {
//     ($ident:ident) => {
//         impl syn::__private::Debug for $ident {
//             fn fmt(&self, f: &mut syn::__private::Formatter) ->
// syn::__private::FmtResult {
// syn::__private::Formatter::write_str(                     f,
//                     syn::__private::concat!("Keyword [",
// syn::__private::stringify!($ident), "]",),                 )
//             }
//         }
//
//         impl syn::__private::Eq for $ident {}
//
//         impl syn::__private::PartialEq for $ident {
//             fn eq(&self, _other: &Self) -> syn::__private::bool {
//                 true
//             }
//         }
//
//         impl syn::__private::Hash for $ident {
//             fn hash<__H: syn::__private::Hasher>(&self, _state: &mut __H) {}
//         }
//     };
// }
//
// // Not public API.
// #[cfg(not(feature = "extra-traits"))]
// #[doc(hidden)]
// #[macro_export]
// macro_rules! impl_extra_traits_for_custom_keyword {
//     ($ident:ident) => {};
// }
