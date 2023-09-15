use std::fmt::Write;
use std::{iter, mem};

use quote::ToTokens;
use quote_use::quote_use as quote;
use syn::parse::discouraged::Speculative;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{braced, bracketed, parenthesized, Ident, Lit, LitStr, Result};

#[allow(non_snake_case)]
fn Ok<T>(t: T) -> Result<T> {
    syn::Result::Ok(t)
}

#[macro_use]
mod macros;

pub enum JsToken {
    Verbatum(String),
    Rust(Ident),
}

pub struct JsTokens(Vec<JsToken>);
impl JsTokens {
    fn verbatum(&mut self, value: impl Into<String>) {
        self.0.push(JsToken::Verbatum(value.into()))
    }

    fn rust(&mut self, value: Ident) {
        self.0.push(JsToken::Rust(value))
    }
}

impl ToTokens for JsTokens {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let mut js_tokens = Vec::new();
        let mut last_verbatum = String::new();
        for token in &self.0 {
            match token {
                JsToken::Verbatum(token) => write!(last_verbatum, " {token}").unwrap(),
                JsToken::Rust(ident) => {
                    let mut last_verbatum = mem::take(&mut last_verbatum);
                    last_verbatum.push(' ');
                    js_tokens.push(quote!($out.push_str(#last_verbatum)));
                    js_tokens.push(quote!($out.push_str(#ident.to_js().as_str())))
                }
            }
        }
        js_tokens.push(quote!($out.push_str(#last_verbatum)));
        quote! {{
            use ::htmx::ToJs as _;
            let mut $out = String::new();
            #(#js_tokens;)*
            $out
        }}
        .to_tokens(tokens)
    }
}

pub trait ToJs {
    fn to_java_script(&self) -> JsTokens {
        let mut s = JsTokens(Vec::new());
        self.to_js(&mut s);
        s
    }

    fn to_js(&self, js: &mut JsTokens);
}

impl ToJs for str {
    fn to_js(&self, js: &mut JsTokens) {
        js.verbatum(self);
    }
}

impl ToJs for Ident {
    fn to_js(&self, js: &mut JsTokens) {
        js.verbatum(self.to_string())
    }
}

impl<T: ToJs> ToJs for &T {
    fn to_js(&self, js: &mut JsTokens) {
        (*self).to_js(js);
    }
}

impl<T: ToJs> ToJs for Option<T> {
    fn to_js(&self, js: &mut JsTokens) {
        if let Some(s) = self.as_ref() {
            s.to_js(js)
        }
    }
}

impl<T: ToJs> ToJs for Punctuated<T, T![,]> {
    fn to_js(&self, js: &mut JsTokens) {
        self.iter().for_each(|t| {
            t.to_js(js);
            ",".to_js(js)
        })
    }
}

impl ToJs for Lit {
    fn to_js(&self, js: &mut JsTokens) {
        // TODO ensure literal valid in js
        self.to_token_stream().to_string().to_js(js)
    }
}

pub struct Script(pub Vec<Stmt>);

impl ToJs for Script {
    fn to_js(&self, js: &mut JsTokens) {
        for stmt in &self.0 {
            stmt.to_js(js);
        }
    }
}

impl Parse for Script {
    fn parse(input: ParseStream) -> Result<Self> {
        iter::from_fn(|| (!input.is_empty()).then(|| input.parse()))
            .collect::<Result<_>>()
            .map(Self)
    }
}

pub enum Stmt {
    Binding(Binding),
    Item(Item),
    Expr(Expr, Option<T![;]>),
}

impl ToJs for Stmt {
    fn to_js(&self, js: &mut JsTokens) {
        match self {
            Stmt::Binding(b) => b.to_js(js),
            Stmt::Item(i) => i.to_js(js),
            Stmt::Expr(e, None) => {
                "return".to_js(js);
                e.to_js(js);
                ";".to_js(js);
            }
            Stmt::Expr(e, Some(_)) => {
                e.to_js(js);
                ";".to_js(js);
            }
        }
    }
}

impl Parse for Stmt {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(T![let]) {
            input.parse().map(Self::Binding)
        } else if input.peek(T![fn]) {
            input.parse().map(Self::Item)
        } else {
            Ok(Self::Expr(input.parse()?, input.parse()?))
        }
    }
}

// Stupid idea, we could consider https://stackoverflow.com/a/16719348/10519515

// TODO: keep in mind, that js allows assigning invalid things sometime
// let [d] = {d: 1} // error
// let {d} = [1] // d == undefined
// let [d] = [1, 2] // 2 is discarded
// let [a, b] = [1] // b == undefined
// I think we should error in all cases, but support rest patterns
pub struct Binding {
    pub let_: T![let],
    pub kind: Option<BindingKind>,
    pub pat: Pat,
    pub init: Option<BindingInit>,
    pub semi: T![;],
}

impl ToJs for Binding {
    fn to_js(&self, js: &mut JsTokens) {
        match self.kind {
            Some(BindingKind::Pub(_)) => "var",
            Some(BindingKind::Mut(_)) => "let",
            None => "const",
        }
        .to_js(js);
        self.pat.to_js(js);
        self.init.to_js(js);
        ";".to_js(js)
    }
}

impl Parse for Binding {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            let_: input.parse()?,
            kind: input.call(BindingKind::parse)?,
            pat: input.parse()?,
            init: (!(input.peek(T![;]) || input.is_empty()))
                .then(|| input.parse())
                .transpose()?,
            semi: input.parse()?,
        })
    }
}

pub enum BindingKind {
    Pub(T![pub]),
    Mut(T![mut]),
}

impl BindingKind {
    fn parse(input: ParseStream) -> Result<Option<Self>> {
        Ok(if input.peek(T![pub]) {
            Some(BindingKind::Pub(input.parse()?))
        } else if input.peek(T![mut]) {
            Some(BindingKind::Mut(input.parse()?))
        } else {
            None
        })
    }
}

pub enum Pat {
    Ident(Ident),
    Tuple(PatTuple),
    Struct(PatStruct),
    // Rest(ColonColon)
}

impl ToJs for Pat {
    fn to_js(&self, js: &mut JsTokens) {
        match self {
            Pat::Ident(i) => i.to_js(js),
            Pat::Tuple(t) => t.to_js(js),
            Pat::Struct(s) => s.to_js(js),
        }
    }
}

impl Parse for Pat {
    fn parse(input: ParseStream) -> Result<Self> {
        input
            .parse()
            .map(Self::Ident)
            .or_else(|_| {
                input
                    .parse()
                    .map(Self::Tuple)
                    .or_else(|_| input.parse().map(Self::Struct))
            })
            .map_err(|_| input.error("Expected ident, `(...)`, or `{..}`"))
    }
}

pub struct PatTuple {
    pub delimiter: TupleDelimiter,
    pub elems: Punctuated<Pat, T![,]>,
    // TODO rest `..`
}

impl ToJs for PatTuple {
    fn to_js(&self, js: &mut JsTokens) {
        "(".to_js(js);
        self.elems.to_js(js);
        ")".to_js(js);
    }
}

impl Parse for PatTuple {
    fn parse(input: ParseStream) -> Result<Self> {
        let elems;
        let delimiter = if input.peek(T![()]) {
            TupleDelimiter::Paren(parenthesized!(elems in input))
        } else if input.peek(T![[]]) {
            TupleDelimiter::Bracket(bracketed!(elems in input))
        } else {
            return Err(input.error("expected `[...]` or `(...)`"));
        };
        Ok(Self {
            delimiter,
            elems: elems.parse_terminated(Pat::parse, T![,])?,
        })
    }
}

pub enum TupleDelimiter {
    Paren(T![()]),
    Bracket(T![[]]),
}

pub struct PatStruct {
    pub brace: T![{}],
    pub fields: Punctuated<FieldPat, T![,]>,
    // TODO rest `..`
}

impl ToJs for PatStruct {
    fn to_js(&self, js: &mut JsTokens) {
        "{".to_js(js);
        self.fields.to_js(js);
        "}".to_js(js);
    }
}

impl Parse for PatStruct {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(PatStruct {
            brace: braced!(content in input),
            fields: content.parse_terminated(FieldPat::parse, T![,])?,
        })
    }
}

pub struct FieldPat {
    pub member: Ident,
    pub pat: Option<(T![:], Box<Pat>)>,
}

impl ToJs for FieldPat {
    fn to_js(&self, js: &mut JsTokens) {
        self.member.to_js(js);
        if let Some((_, pat)) = &self.pat {
            ":".to_js(js);
            pat.to_js(js);
        }
    }
}

impl Parse for FieldPat {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            member: input.parse()?,
            pat: input
                .peek(T![:])
                .then(|| Ok((input.parse()?, input.parse()?)))
                .transpose()?,
        })
    }
}

pub struct BindingInit {
    pub eq: T![=],
    pub expr: Box<Expr>,
    // diverge,
    // I think this could be implemented using a try catch
}

impl ToJs for BindingInit {
    fn to_js(&self, js: &mut JsTokens) {
        "=".to_js(js);
        self.expr.to_js(js);
    }
}

impl Parse for BindingInit {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            eq: input.parse()?,
            expr: input.parse()?,
        })
    }
}

pub enum Expr {
    // Let's be lazy and let js figure out precedence
    Op(Box<Expr>, Op, Box<Expr>),
    Unary(ExprUnary),
    // TODO support template strings, idea: $""
    Lit(Lit),
    Format(T![$], LitStr),
    Block(Block),
    Variable(Ident),
    RustReference(RustReference),
    Paren(ExprParen),
    Call(ExprCall),
    Field(ExprField),
    Tuple(ExprTuple),
    Struct(ExprStruct),
}

impl ToJs for Expr {
    fn to_js(&self, js: &mut JsTokens) {
        match self {
            Expr::Op(l, o, r) => {
                l.to_js(js);
                o.to_js(js);
                r.to_js(js);
            }
            Expr::Unary(u) => u.to_js(js),
            Expr::Lit(l) => l.to_js(js),
            Expr::Format(_, lit) => {
                let lit = lit.value();
                format!("`{}`", lit.replace('`', "\\`")).to_js(js);
            }
            Expr::Block(b) => b.to_js(js),
            Expr::Variable(i) => i.to_js(js),
            Expr::RustReference(r) => r.to_js(js),
            Expr::Paren(p) => p.to_js(js),
            Expr::Call(c) => c.to_js(js),
            Expr::Field(f) => f.to_js(js),
            Expr::Tuple(t) => t.to_js(js),
            Expr::Struct(s) => s.to_js(js),
        }
    }
}

impl Expr {
    fn lhs(input: ParseStream) -> Result<Self> {
        Ok(if input.peek(T![!]) || input.peek(T![-]) {
            Self::Unary(input.parse()?)
        } else if input.peek(Lit) {
            Self::Lit(input.parse()?)
        } else if input.peek(T![$]) && input.peek2(Lit) {
            Self::Format(input.parse()?, input.parse()?)
        } else if input.peek(T![{}]) {
            Self::Block(input.parse()?)
        } else if input.peek(Ident) {
            Self::Variable(input.parse()?)
        } else if input.peek(T![$]) {
            Self::RustReference(input.parse()?)
        } else if input.peek(T![()]) {
            tuple_or_paren(input)?
        } else {
            return Err(input.error("expected expression"));
        })
    }

    fn parse(self, input: ParseStream) -> Result<Self> {
        match () {
            _ if input.is_empty() || input.peek(T![,]) || input.peek(T![;]) => Ok(self),

            _ if input.peek(T![.]) => Self::Field(ExprField {
                expr: self.into(),
                dot: input.parse()?,
                field: input.parse()?,
            })
            .parse(input),

            _ if input.peek(T![()]) => {
                let params;
                Self::Call(ExprCall {
                    expr: self.into(),
                    paren: parenthesized!(params in input),
                    params: Punctuated::parse_terminated(&params)?,
                })
                .parse(input)
            }

            // PRECEDENCE
            _ if Op::peek(input) => Ok(Self::Op(self.into(), input.parse()?, input.parse()?)),

            _ => Err(input.error("expected operator")),
        }
    }
}

impl Parse for Expr {
    fn parse(input: ParseStream) -> Result<Self> {
        Self::lhs(input)?.parse(input)
    }
}

pub enum Op {
    Add(T![+]),
    Sub(T![-]),
    Mul(T![*]),
    Div(T![/]),
    Eq(T![==]),
    Ne(T![!=]),
    Gt(T![>]),
    Ge(T![>=]),
    Lt(T![<]),
    Le(T![<=]),
    And(T![&&]),
    Or(T![||]),
}

impl ToJs for Op {
    fn to_js(&self, js: &mut JsTokens) {
        match self {
            Op::Add(_) => "+",
            Op::Sub(_) => "-",
            Op::Mul(_) => "*",
            Op::Div(_) => "/",
            Op::Eq(_) => "==",
            Op::Ne(_) => "!=",
            Op::Gt(_) => ">",
            Op::Ge(_) => ">=",
            Op::Lt(_) => "<",
            Op::Le(_) => "<=",
            Op::And(_) => "&&",
            Op::Or(_) => "||",
        }
        .to_js(js)
    }
}

impl Op {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(T![+])
            || input.peek(T![-])
            || input.peek(T![*])
            || input.peek(T![/])
            || input.peek(T![==])
            || input.peek(T![!=])
            || input.peek(T![>])
            || input.peek(T![>])
            || input.peek(T![<])
            || input.peek(T![<=])
            || input.peek(T![&&])
            || input.peek(T![||])
    }
}

impl Parse for Op {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(match () {
            _ if input.peek(T![+]) => Self::Add(input.parse()?),
            _ if input.peek(T![-]) => Self::Sub(input.parse()?),
            _ if input.peek(T![*]) => Self::Mul(input.parse()?),
            _ if input.peek(T![/]) => Self::Div(input.parse()?),
            _ if input.peek(T![==]) => Self::Eq(input.parse()?),
            _ if input.peek(T![!=]) => Self::Ne(input.parse()?),
            _ if input.peek(T![>]) => Self::Gt(input.parse()?),
            _ if input.peek(T![>]) => Self::Gt(input.parse()?),
            _ if input.peek(T![<]) => Self::Lt(input.parse()?),
            _ if input.peek(T![<=]) => Self::Le(input.parse()?),
            _ if input.peek(T![&&]) => Self::And(input.parse()?),
            _ if input.peek(T![||]) => Self::Or(input.parse()?),
            _ => return Err(input.error("expected operator")),
        })
    }
}

pub struct ExprUnary {
    pub op: UnaryOp,
    pub expr: Box<Expr>,
}

impl ToJs for ExprUnary {
    fn to_js(&self, js: &mut JsTokens) {
        match self.op {
            UnaryOp::Not(_) => "!",
            UnaryOp::Neg(_) => "-",
        }
        .to_js(js);
        self.expr.to_js(js);
    }
}

impl Parse for ExprUnary {
    fn parse(input: ParseStream) -> Result<Self> {
        // PRECEDENCE: this would result in parsing `!a || b` as `!(a || b)`
        Ok(Self {
            op: input.parse()?,
            expr: input.parse()?,
        })
    }
}

pub enum UnaryOp {
    Not(T![!]),
    Neg(T![-]),
}

impl Parse for UnaryOp {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse().map(Self::Not).or_else(|_| {
            input
                .parse()
                .map(Self::Neg)
                .map_err(|_| input.error("Expected `!` or `-`"))
        })
    }
}

pub struct Block {
    pub braces: T![{}],
    pub stmts: Vec<Stmt>,
}

impl ToJs for Block {
    fn to_js(&self, js: &mut JsTokens) {
        "{".to_js(js);
        self.stmts.iter().for_each(|s| s.to_js(js));
        "}".to_js(js);
    }
}

impl Parse for Block {
    fn parse(input: ParseStream) -> Result<Self> {
        let stmts;
        Ok(Self {
            braces: braced!(stmts in input),
            stmts: Script::parse(&stmts)?.0,
        })
    }
}

pub struct RustReference {
    pub dollar: T![$],
    pub ident: Ident,
}

impl ToJs for RustReference {
    fn to_js(&self, js: &mut JsTokens) {
        js.rust(self.ident.clone())
    }
}

impl Parse for RustReference {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            dollar: input.parse()?,
            ident: input.parse()?,
        })
    }
}

pub struct ExprParen {
    pub paren: T![()],
    pub expr: Box<Expr>,
}

impl ToJs for ExprParen {
    fn to_js(&self, js: &mut JsTokens) {
        "(".to_js(js);
        self.expr.to_js(js);
        ")".to_js(js);
    }
}

pub struct ExprTuple {
    pub delimiter: TupleDelimiter,
    pub fields: Punctuated<Expr, T![,]>,
}

impl ToJs for ExprTuple {
    fn to_js(&self, js: &mut JsTokens) {
        "[".to_js(js);
        self.fields.to_js(js);
        "]".to_js(js);
    }
}

fn tuple_or_paren(input: ParseStream) -> Result<Expr> {
    let content;
    let delimiter = if input.peek(T![()]) {
        let paren = parenthesized!(content in input);
        let fork = content.fork();
        let expr = fork.parse()?;
        if input.is_empty() {
            content.advance_to(&fork);
            return Ok(Expr::Paren(ExprParen {
                paren,
                expr: Box::new(expr),
            }));
        } else if input.peek(T![,]) {
            TupleDelimiter::Paren(paren)
        } else {
            return Err(content.error("expected `,` or operator"));
        }
    } else if input.peek(T![[]]) {
        TupleDelimiter::Bracket(bracketed!(content in input))
    } else {
        return Err(input.error("expected `[...]` or `(...)`"));
    };
    Ok(Expr::Tuple(ExprTuple {
        delimiter,
        fields: Punctuated::parse_terminated(input)?,
    }))
}

impl Parse for ExprParen {
    fn parse(input: ParseStream) -> Result<Self> {
        let expr;
        Ok(Self {
            paren: parenthesized!(expr in input),
            expr: expr.parse()?,
        })
    }
}

pub struct ExprCall {
    pub expr: Box<Expr>,
    pub paren: T![()],
    pub params: Punctuated<Expr, T![,]>,
}

impl ToJs for ExprCall {
    fn to_js(&self, js: &mut JsTokens) {
        self.expr.to_js(js);
        "(".to_js(js);
        self.params.to_js(js);
        ")".to_js(js);
    }
}

pub struct ExprField {
    pub expr: Box<Expr>,
    pub dot: T![.],
    pub field: Ident,
}

impl ToJs for ExprField {
    fn to_js(&self, js: &mut JsTokens) {
        self.expr.to_js(js);
        ".".to_js(js);
        self.field.to_js(js);
    }
}

pub struct ExprStruct {
    pub brace: T![{}],
    pub fields: Punctuated<(Ident, T![:], Expr), T![,]>,
}

impl ToJs for ExprStruct {
    fn to_js(&self, js: &mut JsTokens) {
        "{".to_js(js);
        for (ident, _, expr) in &self.fields {
            ident.to_js(js);
            ":".to_js(js);
            expr.to_js(js);
            ",".to_js(js);
        }
        "}".to_js(js);
    }
}

pub enum Item {
    Fn(Fn),
}

impl ToJs for Item {
    fn to_js(&self, js: &mut JsTokens) {
        let Self::Fn(fun) = self;
        fun.to_js(js);
    }
}

impl Parse for Item {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse().map(Self::Fn)
    }
}

pub struct Fn {
    pub fn_: T![fn],
    pub name: Ident,
    pub paren: T![()],
    pub params: Punctuated<Ident, T![,]>,
    pub body: Block,
}

impl ToJs for Fn {
    fn to_js(&self, js: &mut JsTokens) {
        "function".to_js(js);
        self.name.to_js(js);
        "(".to_js(js);
        self.params.to_js(js);
        ")".to_js(js);
        self.body.to_js(js);
    }
}

impl Parse for Fn {
    fn parse(input: ParseStream) -> Result<Self> {
        let params;
        Ok(Self {
            fn_: input.parse()?,
            name: input.parse()?,
            paren: parenthesized!(params in input),
            params: Punctuated::parse_terminated(&params)?,
            body: input.parse()?,
        })
    }
}

#[test]
fn basic() -> syn::Result<()> {
    use quote::quote;
    use syn::parse2;
    let rust = quote! {
        fn on_click(event) {
            // TODO support rust in template strings
            let name = $name;
            console.log($name);
            alert($"Hi ${name} you triggered an event ${event.type}");
        }
    };
    let ast: Script = parse2(rust)?;
    insta::assert_snapshot!(ast.to_java_script().to_token_stream().to_string());
    Ok(())
}
