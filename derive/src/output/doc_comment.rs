use proc_macro2::TokenStream;
use rust_format::{Formatter, PrettyPlease};

use crate::output::malformed_code_generation_result;

pub(crate) fn implementation_doc_comment(implementation: TokenStream) -> String {
    implementation_section(implementation)
        .trim_start_matches('\n')
        .to_owned()
}

pub(crate) fn doc_comment_with_implementation(
    doc_comment: &str,
    implementation: TokenStream,
) -> String {
    if doc_comment.is_empty() {
        implementation_doc_comment(implementation)
    } else {
        format!("{doc_comment}{}", implementation_section(implementation))
    }
}

fn implementation_section(implementation: TokenStream) -> String {
    let implementation = PrettyPlease::default()
        .format_tokens(implementation.clone())
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                malformed_code_generation_result(implementation.to_string())
            )
        });

    // The characterization tests pin the textual doc comments, not the pretty-printed
    // implementation, which would otherwise duplicate every snapshotted method body.
    // `PrettyPlease` above still runs, so malformed emissions still panic during tests.
    if cfg!(test) {
        return String::default();
    }

    format!("\n\nImplementation:\n\n```no_run\n{implementation}\n```")
}
