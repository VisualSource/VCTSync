use crate::util::strip_braces;
use quote::{ToTokens, TokenStreamExt};
use rstml::node::{AttributeValueExpr, KeyedAttribute, NodeAttribute};
use std::collections::HashMap;
pub struct Attributes<'a>(HashMap<String, &'a KeyedAttribute>);

impl<'a> Attributes<'a> {
    pub fn new(attributes: &'a [NodeAttribute]) -> Self {
        let mut attrs = HashMap::new();

        for attr in attributes {
            match attr {
                NodeAttribute::Block(_) => unimplemented!(),
                NodeAttribute::Attribute(keyed_attribute) => {
                    let key = keyed_attribute.key.to_string();
                    attrs.insert(key, keyed_attribute);
                }
            }
        }

        Self(attrs)
    }

    pub fn append_if_value(
        &self,
        stream: &mut proc_macro2::TokenStream,
        key: &str,
        tokens: fn(proc_macro2::TokenStream) -> proc_macro2::TokenStream,
    ) -> syn::Result<()> {
        if let Some(attr) = self.0.get(key) {
            if let Some(value) = attr.possible_value.to_value() {
                let value_stream = strip_braces(value.value.to_token_stream())?;

                stream.append_all(tokens(value_stream));
            }
        }

        Ok(())
    }

    pub fn has_attr(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }

    pub fn get_value(&self, key: &str) -> Option<&'a AttributeValueExpr> {
        if let Some(r) = self.0.get(key) {
            r.possible_value.to_value()
        } else {
            None
        }
    }

    pub fn append_if(
        &self,
        stream: &mut proc_macro2::TokenStream,
        key: &str,
        tokens: fn() -> proc_macro2::TokenStream,
    ) {
        if self.0.contains_key(key) {
            stream.append_all(tokens());
        }
    }
}
