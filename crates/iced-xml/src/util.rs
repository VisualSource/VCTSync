use proc_macro2::{Delimiter, TokenTree};

pub fn strip_braces(stream: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let mut iter = stream.clone().into_iter(); // ref count vec

    let (Some(TokenTree::Group(group)), None) = (iter.next(), iter.next()) else {
        return Ok(stream);
    };

    if group.delimiter() != Delimiter::Brace {
        return Ok(stream);
    }

    let inner = group.stream();
    if inner.is_empty() {
        return Err(syn::Error::new(group.span(), "expected an expression"));
    }

    let has_stmt = inner
        .clone()
        .into_iter()
        .any(|t| matches!(&t, TokenTree::Punct(p) if p.as_char() == ';'));

    if has_stmt {
        return Err(syn::Error::new(
            group.span(),
            "expected a single expression, found statements",
        ));
    }

    Ok(inner)
}
