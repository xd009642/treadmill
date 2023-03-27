//! Useful resources when implementing this:
//!
//! * https://ferrous-systems.com/blog/testing-proc-macros/#proc-macro-refresher
//! * https://github.com/dtolnay/syn/blob/master/examples/trace-var/trace-var/src/lib.rs
use proc_macro::TokenStream;
use quote::quote;
use syn::{fold::Fold, parse_macro_input, Ident, ItemFn};

/// This is used to take in the ident from a function and rename it to have `treadmill_` as a
/// prefix.
///
/// TODO see why Fold trait is necessary? Does it fix the span locations or is that just done by
/// quote! at the end? Things don't really seem clear on why you edit it this way instead of just
/// doing it directly and the docs are completely opaque to me. I just kind of stumbled around
/// blindly trying to figure this out
struct TreadmillPrefixRenamer;

impl Fold for TreadmillPrefixRenamer {
    fn fold_ident(&mut self, i: Ident) -> Ident {
        let new_name = format!("treadmill_{}", i);
        Ident::new(&new_name, i.span())
    }
}

// if we could translate
//
// #[treadmill::main]
// async fn main(T..) -> R {
//     //block
// }
//
// into
//
// async fn treadmill_main(T..) -> R {
//
// }
//
//
// fn main(T..) -> R {
//     let rt = Runtime::default();
//     rt.block_on(treadmill_main(T..))
// }
//
// Then that should handle things nicely (in theory)

#[proc_macro_attribute]
pub fn main(args: TokenStream, item: TokenStream) -> TokenStream {
    asyncify_function(args, item, false)
}

#[proc_macro_attribute]
pub fn test(args: TokenStream, item: TokenStream) -> TokenStream {
    asyncify_function(args, item, true)
}

fn asyncify_function(args: TokenStream, item: TokenStream, is_test: bool) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemFn);

    let mut output = input.sig.clone();
    output.asyncness = None;

    input.sig.ident = TreadmillPrefixRenamer.fold_ident(input.sig.ident);

    let test_attr = if is_test {
        quote! {#[test]}
    } else {
        quote! {}
    };

    let new_name = &input.sig.ident;

    let res = TokenStream::from(quote! {
        #input

        #test_attr
        #output {
            let rt = treadmill::Runtime::default();
            rt.block_on(#new_name())
        }
    });
    res
}
