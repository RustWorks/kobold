use proc_macro::{TokenStream, TokenTree, Delimiter, Span};
use proc_macro::token_stream::IntoIter as TokenIter;
use quote::{quote, quote_spanned};
use crate::dom::{Node, Element};

#[derive(Debug)]
pub struct ParseError {
    msg: String,
    span: Span,
    tt: Option<TokenTree>,
}

impl ParseError {
    pub fn new<S: Into<String>>(msg: S, span: Span) -> Self {
        ParseError {
            msg: msg.into(),
            span,
            tt: None
        }
    }

    pub fn from_tt<S: Into<String>>(msg: S, tt: Option<TokenTree>) -> Self {
        let mut error = ParseError::from(tt);

        error.msg = msg.into();
        error
    }

    pub fn tokenize(self) -> TokenStream {
        let msg = self.msg;
        (quote_spanned! { self.span.into() =>
            fn _parse_error() {
                compile_error!(#msg)
            }
        }).into()
    }
}

impl From<Option<TokenTree>> for ParseError {
    fn from(tt: Option<TokenTree>) -> Self {
        let span = tt.as_ref().map(|tt| tt.span()).unwrap_or_else(|| Span::call_site());

        ParseError {
            msg: "Unexpected token".into(),
            span,
            tt,
        }
    }
}

pub fn parse(tokens: TokenStream) -> Result<Node, ParseError> {
    let mut iter = tokens.into_iter();

    let node = parse_node(&mut iter)?;

    // Convert to fragment if necessary
    match parse_node(&mut iter) {
        Ok(second) => {
            let mut fragment = vec![node, second];

            loop {
                match parse_node(&mut iter) {
                    Ok(node) => fragment.push(node),
                    Err(err) if err.tt.is_none() => break,
                    err => return err,
                }
            }

            Ok(Node::Fragment(fragment))
        },
        Err(err) if err.tt.is_none() => Ok(node),
        err => err,
    }
}

fn parse_node(iter: &mut TokenIter) -> Result<Node, ParseError> {
    match iter.next() {
        Some(TokenTree::Punct(punct)) if punct.as_char() == '<' => {
            Ok(Node::Element(parse_element(iter)?))
        },
        Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
            Ok(Node::Expression(group.stream().into()))
        },
        Some(TokenTree::Literal(lit)) => {
            let stringified = lit.to_string();

            let mut chars = stringified.chars();

            let tokens = match chars.next() {
                // Take the string verbatim
                Some('"') | Some('\'') => TokenStream::from(TokenTree::Literal(lit)).into(),
                _ => quote!(#stringified),
            };

            Ok(Node::Text(tokens))
        },
        tt => Err(ParseError::from_tt("Expected an element, {expression}, or a string literal", tt)),
    }
}

fn parse_element(iter: &mut TokenIter) -> Result<Element, ParseError> {
    let (tag, _) = expect_ident(iter.next())?;

    let mut element = Element {
        tag,
        props: Vec::new(),
        children: Vec::new(),
    };

    // Props loop
    loop {
        match iter.next() {
            Some(TokenTree::Ident(key)) => {
                let key = key.to_string();

                expect_punct(iter.next(), '=')?;

                match iter.next() {
                    Some(value) => {
                        element.props.push((key, TokenStream::from(value).into()));
                    },
                    tt => return Err(tt.into()),
                }
            },
            Some(TokenTree::Punct(punct)) if punct.as_char() == '/' => {
                expect_punct(iter.next(), '>')?;

                // Self-closing tag, no need to parse further
                return Ok(element);
            },
            Some(TokenTree::Punct(punct)) if punct.as_char() == '>' => {
                break;
            },
            tt => return Err(ParseError::from_tt("Expected identifier, /, or >", tt))
        }
    }

    // Children loop
    loop {
        match parse_node(iter) {
            Ok(child) => element.children.push(child),
            Err(err) => match err.tt {
                Some(TokenTree::Punct(punct)) if punct.as_char() == '/' => break,
                _ => return Err(err),
            },
        }
    }

    let (closing, closing_span) = expect_ident(iter.next())?;

    if closing != element.tag {
        return Err(ParseError::new(
            format!("Expected a closing tag for {}, but got {} instead", element.tag, closing),
            closing_span,
        ));
    }

    expect_punct(iter.next(), '>')?;

    Ok(element)
}

fn expect_punct(tt: Option<TokenTree>, expect: char) -> Result<(), ParseError> {
    match tt {
        Some(TokenTree::Punct(punct)) if punct.as_char() == expect => Ok(()),
        tt => Err(ParseError::from_tt(format!("Expected {}", expect), tt)),
    }
}

fn expect_ident(tt: Option<TokenTree>) -> Result<(String, Span), ParseError> {
    match tt {
        Some(TokenTree::Ident(ident)) => Ok((ident.to_string(), ident.span())),
        tt => Err(ParseError::from_tt("Expected identifier", tt)),
    }
}
