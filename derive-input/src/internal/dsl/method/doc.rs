//! The pieces every generated rustdoc text is assembled from: paragraphs, sections, and code
//! the way a person writes it.

use {
    proc_macro2::{Delimiter, Group, Punct, Spacing, TokenStream, TokenTree},
    std::borrow::Borrow,
};

/// The non-empty `parts` as paragraphs, each separated from the next by a blank line.
pub fn paragraphs<Part: Borrow<str>>(parts: impl IntoIterator<Item = Part>) -> String {
    parts
        .into_iter()
        .filter(|part| !part.borrow().is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// A doc comment section: its heading, a lead sentence when there is one, and its bullets.
/// Nothing when there are no bullets.
pub fn section(heading: &str, lead: Option<&str>, bullets: &[String]) -> String {
    if bullets.is_empty() {
        return String::new();
    }

    let lead = lead.map(|lead| format!("{lead}\n\n")).unwrap_or_default();

    format!("# {heading}\n\n{lead}{}", bullets.join("\n"))
}

/// Tokens the way a person writes them, for documentation.
///
/// `proc_macro2` puts a space between every two tokens, as in `String :: new ()`. This keeps
/// a space only between two words, around an operator between two operands, and after `,`,
/// `;` and a single `:`, so the tokens read `String::new()`, `vec![1, 2]`, `-1` or `a + b`.
/// A literal keeps its own text, a space inside a string literal included. Generic arguments
/// keep the spaces around their angle brackets, as in `Vec::< u8 >::new()`.
pub fn written_tokens(tokens: TokenStream) -> String {
    let mut written = String::new();
    write_tokens(tokens, &mut written, &mut Previous::Start);

    written
}

/// The kind of the token written last, which decides whether a space goes in front of the
/// next one.
#[derive(Clone, Copy, PartialEq)]
enum Previous {
    /// Nothing yet: the start of the tokens, or of the inside of a bracket.
    Start,
    /// A word, a literal, a closing bracket or a `?`: the end of an operand.
    Operand,
    /// A token the next one follows without a space: `.`, the second `:` of `::`, the `!` of
    /// a macro, or an operator in front of its operand, such as the `-` of `-1`.
    Glue,
    /// The first `:` of `::`.
    FirstColon,
    /// The first character of an operator of several, such as the first `=` of `==`.
    OperatorStart,
    /// A binary operator, `,`, `;` or a single `:`, which a space follows.
    Separator,
}

impl Previous {
    /// Whether a space separates this token from an operand after it.
    fn is_spaced_from_an_operand(self) -> bool {
        matches!(self, Previous::Operand | Previous::Separator)
    }
}

fn write_tokens(tokens: TokenStream, written: &mut String, previous: &mut Previous) {
    for token in tokens {
        match token {
            TokenTree::Ident(ident) => write_operand(&ident.to_string(), written, previous),
            TokenTree::Literal(literal) => write_operand(&literal.to_string(), written, previous),
            TokenTree::Punct(punct) => write_punctuation(&punct, written, previous),
            TokenTree::Group(group) => write_group(&group, written, previous),
        }
    }
}

fn write_operand(text: &str, written: &mut String, previous: &mut Previous) {
    if previous.is_spaced_from_an_operand() {
        written.push(' ');
    }

    written.push_str(text);
    *previous = Previous::Operand;
}

fn write_punctuation(punct: &Punct, written: &mut String, previous: &mut Previous) {
    let character = punct.as_char();
    let starts_an_operator = punct.spacing() == Spacing::Joint;
    let follows_an_operand = *previous == Previous::Operand;
    let operator = match starts_an_operator {
        true => Previous::OperatorStart,
        false => Previous::Separator,
    };

    let (is_spaced, next) = match character {
        ',' | ';' => (false, Previous::Separator),
        '.' => (false, Previous::Glue),
        '?' => (false, Previous::Operand),
        ':' if *previous == Previous::FirstColon => (false, Previous::Glue),
        ':' if starts_an_operator => (false, Previous::FirstColon),
        ':' => (false, Previous::Separator),
        _ if *previous == Previous::OperatorStart => (false, operator),
        '!' if follows_an_operand && !starts_an_operator => (false, Previous::Glue),
        '!' | '-' | '&' | '*' if !follows_an_operand => {
            (previous.is_spaced_from_an_operand(), Previous::Glue)
        }
        _ => (
            follows_an_operand || *previous == Previous::Separator,
            operator,
        ),
    };

    if is_spaced {
        written.push(' ');
    }

    written.push(character);
    *previous = next;
}

fn write_group(group: &Group, written: &mut String, previous: &mut Previous) {
    let delimiter = group.delimiter();

    if delimiter == Delimiter::None {
        write_tokens(group.stream(), written, previous);
        return;
    }

    // A call, an index and a macro take their brackets right after what they follow.
    let follows_its_callee =
        delimiter != Delimiter::Brace && matches!(*previous, Previous::Operand | Previous::Glue);

    if !follows_its_callee && previous.is_spaced_from_an_operand() {
        written.push(' ');
    }

    let mut inside = String::new();
    write_tokens(group.stream(), &mut inside, &mut Previous::Start);

    let bracketed = match delimiter {
        Delimiter::Parenthesis => format!("({inside})"),
        Delimiter::Bracket => format!("[{inside}]"),
        _ if inside.is_empty() => "{}".to_string(),
        _ => format!("{{ {inside} }}"),
    };

    written.push_str(&bracketed);
    *previous = Previous::Operand;
}
