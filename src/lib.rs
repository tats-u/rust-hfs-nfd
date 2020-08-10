mod code_table;
#[cfg(test)]
mod code_table_test;
mod reverse_tree;
use code_table::{MAP_TO_HFS, MAP_TO_NORMAL};
use reverse_tree::ReverseTreeNode;
use std::borrow::Cow;

/// Applies the Unicode decomposition similar to NFD used in HFS+
///
/// # Arguments
///
/// * `input` - A string to be decomposed
/// 
/// # Examples
/// 
/// ```
/// use hfs_nfd::decompose_into_hfs_nfd;
/// assert_eq!(&decompose_into_hfs_nfd("Pok\u{00E9}mon"), "Poke\u{0301}mon");
/// ```
pub fn decompose_into_hfs_nfd<'a, S: Into<Cow<'a, str>>>(input: S) -> String {
    let input = input.into();
    let mut result = String::new();

    for c in input.chars() {
        match MAP_TO_HFS.get(&c) {
            None => result.push(c),
            Some(&decomposed) => result += decomposed,
        }
    }
    return result;
}

/// Restores a commonly encoded string from one applied the Unicode decomposition similar to NFS used in HFS+ to
/// 
/// # Arguments
/// 
/// * `input` - A string to be restored from
/// 
/// # Examples
/// 
/// ```
/// use hfs_nfd::compose_from_hfs_nfd;
/// assert_eq!(&compose_from_hfs_nfd("Poke\u{0301}mon"), "Pok\u{00E9}mon");
/// ```
pub fn compose_from_hfs_nfd<'a, S: Into<Cow<'a, str>>>(input: S) -> String {
    let input = input.into();
    let mut result = String::new();
    let mut referencing_dict = &*MAP_TO_NORMAL;
    let mut pending_chars = String::new();
    let mut tentative_determined_chars: Option<Box<String>> = None;
    let mut tentative_composed = None;

    for c in input.chars() {
        loop {
            match referencing_dict.get(&c) {
                None
                | Some(ReverseTreeNode {
                    current: None,
                    next: None,
                }) => {
                    let mut try_again = false;
                    if let Some(ch) = tentative_composed {
                        result.push(ch);
                        tentative_composed = None;
                        try_again = true;
                        tentative_determined_chars = None;
                    }
                    if !pending_chars.is_empty() {
                        result += &pending_chars;
                        try_again = true;
                        pending_chars.clear();
                    }
                    referencing_dict = &*MAP_TO_NORMAL;
                    if try_again {
                        continue;
                    }
                    result.push(c);
                    break;
                }
                Some(ReverseTreeNode {
                    current: None,
                    next: Some(sub_dict),
                }) => {
                    referencing_dict = sub_dict.as_ref();
                    pending_chars.push(c);
                    break;
                }
                Some(ReverseTreeNode {
                    current: Some(composed_char),
                    next: Some(sub_dict),
                }) => {
                    referencing_dict = sub_dict.as_ref();
                    tentative_composed = Some(*composed_char);
                    match tentative_determined_chars.as_mut() {
                        Some(existing_chars) => {
                            existing_chars.push(c);
                        }
                        None => {
                            pending_chars.push(c);
                            tentative_determined_chars = Some(Box::from(pending_chars.clone()));
                            pending_chars.clear();
                        }
                    }
                    break;
                }
                Some(ReverseTreeNode {
                    current: Some(composed_char),
                    next: None,
                }) => {
                    pending_chars.clear();
                    result.push(*composed_char);
                    tentative_composed = None;
                    tentative_determined_chars = None;
                    referencing_dict = &*MAP_TO_NORMAL;
                    break;
                }
            }
        }
    }
    if let Some(c) = tentative_composed {
        result.push(c);
    }
    if !pending_chars.is_empty() {
        result += &pending_chars;
    }
    return result;
}

#[cfg(test)]
mod tests {
    use super::*;
    use lazy_static::lazy_static;
    lazy_static! {
        static ref EXAMINEE: Vec<(&'static str, &'static str)> = vec![
            ("Pokémonポケモン", "Pokémonポケモン"),
            ("ポプテピピック", "ポプテピピック"),
            (
                "ポプテピピックボブネミミッミ",
                "ポプテピピックボブネミミッミ",
            ),
            ("ボブ・サップ", "ボブ・サップ"),
            (
                "ḹ₥קσƧƨῗɓŁḕ",
                "\u{006C}\u{0323}\u{0304}₥קσƧƨ\u{03B9}\u{0308}\u{0342}ɓŁ\u{0065}\u{0304}\u{0300}"
            ),
            ("σου πονηρὸς ᾖ ὅλον τὸ", "σου πονηρ\u{03BF}\u{0300}ς \u{03B7}\u{0345}\u{0313}\u{0342} \u{03BF}\u{0314}\u{0301}λον τ\u{03BF}\u{0300}"),
            ("Université", "Université"),
            ("D:\\‰{'é“H'ê\\−w›ï”'\\fiúŒ{ŠÕ'°ŒÆ›u−w›ï\\vol.27-no.5 ... - J-Stage", "D:\\‰{\'e\u{301}“H\'e\u{302}\\−w›i\u{308}”\'\\fiu\u{301}Œ{S\u{30c}O\u{303}\'°ŒÆ›u−w›i\u{308}\\vol.27-no.5 ... - J-Stage")
        ];
        static ref EXAMINEE_IMMUTABLE: Vec<&'static str> = vec![
            "Immutable",
            "Can't be changed",
            "かわらない",
            "ヘンカナシ",
            "不変",
        ];
    }
    #[test]
    fn decompose_fixed_strings_test() {
        for (composed, decomposed) in EXAMINEE.iter() {
            let converted = decompose_into_hfs_nfd(*composed);
            assert_eq!(&converted, *decomposed);
        }
    }
    #[test]
    fn compose_from_fixed_strings_test() {
        for (composed, decomposed) in EXAMINEE.iter() {
            let converted = compose_from_hfs_nfd(*decomposed);
            assert_eq!(&converted, *composed);
        }
    }
    #[test]
    fn compose_already_composed_fixed_strings_test() {
        for (composed, _) in EXAMINEE.iter() {
            let converted = compose_from_hfs_nfd(*composed);
            assert_eq!(&converted, *composed);
        }
    }
    #[test]
    fn decompose_already_deomposed_identity_fixed_strings_test() {
        for (_, decomposed) in EXAMINEE.iter() {
            let converted = decompose_into_hfs_nfd(*decomposed);
            assert_eq!(&converted, *decomposed);
        }
    }
    #[test]
    fn compose_immutable_fixed_strings_test() {
        for s in EXAMINEE_IMMUTABLE.iter() {
            let converted = compose_from_hfs_nfd(*s);
            assert_eq!(&converted, *s);
        }
    }
    #[test]
    fn decompose_immutable_fixed_strings_test() {
        for s in EXAMINEE_IMMUTABLE.iter() {
            let converted = decompose_into_hfs_nfd(*s);
            assert_eq!(&converted, *s);
        }
    }
}
