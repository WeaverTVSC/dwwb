use std::path::Path;

use lazy_static::lazy_static;
use regex::Regex;

pub fn path_to_url<P: AsRef<Path>>(path: P) -> String {
    let path = path.as_ref();

    let mut it = path.components();
    if let Some(comp) = it.next() {
        let mut output = String::from(
            comp.as_os_str()
                .to_str()
                .expect("Invalid UTF in input path"),
        );
        for comp in it {
            output += "/";
            output += comp
                .as_os_str()
                .to_str()
                .expect("Invalid UTF in input path");
        }
        output
    } else {
        String::new()
    }
}

/// Transforms the given text to title case
pub fn title_case<S: AsRef<str>>(input: &S) -> String {
    lazy_static! {
        static ref WORD_START_REGEX: Regex = Regex::new(r"(?:^|\b)(\w)").unwrap();
    }

    WORD_START_REGEX
        .replace_all(input.as_ref(), |captures: &regex::Captures| {
            captures.get(1).unwrap().as_str().to_uppercase()
        })
        .to_string()
}
