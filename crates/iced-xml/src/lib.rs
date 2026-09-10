//! Iced XML
//!    
//! Write iced ui element using xml like syntax
//!
//!
//! ```
//!  ui!{
//!     <view>
//!         <text>Hello</text>
//!     </view>
//! }
//! ```

mod attribute;
mod macros;
mod parser;
mod util;

#[proc_macro]
pub fn ui(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);

    match parser::parse_xml(input) {
        Ok(out) => out.into(),
        Err(err) => err.into_compile_error().into(),
    }
}
