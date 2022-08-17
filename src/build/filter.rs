use std::borrow::Cow;

use lazy_static::lazy_static;
use regex::Regex;

pub fn make_pandoc_filter(root_url: String) -> impl Fn(String) -> String {
    lazy_static! {
        static ref ROOT_REGEX: Regex = Regex::new("%ROOT%/?").unwrap();
    }

    move |mut s| {
        if let Cow::Owned(replaced) = ROOT_REGEX.replace_all(&s, &root_url) {
            s = replaced
        }
        s
    }
}
