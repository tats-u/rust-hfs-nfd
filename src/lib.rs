#![cfg_attr(feature = "bench", feature(test))]
#[cfg(feature = "bench")]
extern crate test;
mod code_table;
#[cfg(test)]
mod code_table_test;
mod hangul;
mod reverse_tree;
use code_table::{MAP_TO_HFS, MAP_TO_NORMAL};
use hangul::{
    compose_hangul_jamos, decomopse_hangul_syllable, is_hangul_conjoinable_jamo,
    is_hangul_precomposed_syllable,
};
use reverse_tree::ReverseTreeNode;

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
pub fn decompose_into_hfs_nfd(input: &str) -> String {
    let mut result = String::new();

    for c in input.chars() {
        match MAP_TO_HFS.get(&c) {
            None => {
                if is_hangul_precomposed_syllable(c) {
                    result += &decomopse_hangul_syllable(c).into_boxed_str();
                } else {
                    result.push(c);
                }
            }
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
pub fn compose_from_hfs_nfd(input: &str) -> String {
    let mut result = String::new();
    let mut referencing_dict = &*MAP_TO_NORMAL;
    let mut pending_chars = String::new();
    let mut pending_hangul_jamos = String::new();
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
                        if !pending_hangul_jamos.is_empty() {
                            result += &compose_hangul_jamos(&pending_hangul_jamos).into_boxed_str();
                            pending_hangul_jamos.clear();
                        }
                        result.push(ch);
                        tentative_composed = None;
                        try_again = true;
                        tentative_determined_chars = None;
                    }
                    if !pending_chars.is_empty() {
                        if !pending_hangul_jamos.is_empty() {
                            result += &compose_hangul_jamos(&pending_hangul_jamos).into_boxed_str();
                            pending_hangul_jamos.clear();
                        }
                        result += &pending_chars;
                        try_again = true;
                        pending_chars.clear();
                    }
                    referencing_dict = &*MAP_TO_NORMAL;
                    if try_again {
                        continue;
                    }
                    // Out of the Apple's table

                    // Korean hangul jamo
                    if is_hangul_conjoinable_jamo(c) {
                        pending_hangul_jamos.push(c);
                    } else {
                        if !pending_hangul_jamos.is_empty() {
                            result += &compose_hangul_jamos(&pending_hangul_jamos).into_boxed_str();
                            pending_hangul_jamos.clear();
                        }
                        result.push(c);
                    }
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
                    if !pending_hangul_jamos.is_empty() {
                        result += &compose_hangul_jamos(&pending_hangul_jamos).into_boxed_str();
                        pending_hangul_jamos.clear();
                    }
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
    if !pending_hangul_jamos.is_empty() {
        result += &compose_hangul_jamos(&pending_hangul_jamos).into_boxed_str();
    }
    if !pending_chars.is_empty() {
        result += &pending_chars;
    }
    return result;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "bench")]
    use test::Bencher;
    static EXAMINEE: &[(&'static str, &'static str)] = &[
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
            ("D:\\‰{'é“H'ê\\−w›ï”'\\fiúŒ{ŠÕ'°ŒÆ›u−w›ï\\vol.27-no.5 ... - J-Stage", "D:\\‰{\'e\u{301}“H\'e\u{302}\\−w›i\u{308}”\'\\fiu\u{301}Œ{S\u{30c}O\u{303}\'°ŒÆ›u−w›i\u{308}\\vol.27-no.5 ... - J-Stage"),
            ("チョイ・ボンゲ(Choi Bounge / 최번개)", "チョイ・ボンゲ(Choi Bounge / 최번개)"),
            ("ハン・ジュリ(Han Juri / 한주리)", "ハン・ジュリ(Han Juri / 한주리)"),
            ("チョイ・ボンゲ최번개ハン・ジュリ한주리", "チョイ・ボンゲ최번개ハン・ジュリ한주리"),
            ("か카ka아a에éゲ게gé", "か카ka아a에éゲ게gé")
        ];
    static EXAMINEE_IMMUTABLE: &[&'static str] = &[
        "Immutable",
        "Can't be changed",
        "かわらない",
        "ヘンカナシ",
        "不変",
    ];
    #[test]
    fn decompose_fixed_strings_test() {
        for (composed, decomposed) in EXAMINEE {
            let converted = decompose_into_hfs_nfd(*composed);
            assert_eq!(&converted, *decomposed);
        }
    }
    #[test]
    fn compose_from_fixed_strings_test() {
        for (composed, decomposed) in EXAMINEE {
            let converted = compose_from_hfs_nfd(*decomposed);
            assert_eq!(&converted, *composed);
        }
    }
    #[test]
    fn compose_already_composed_fixed_strings_test() {
        for (composed, _) in EXAMINEE {
            let converted = compose_from_hfs_nfd(*composed);
            assert_eq!(&converted, *composed);
        }
    }
    #[test]
    fn decompose_already_deomposed_identity_fixed_strings_test() {
        for (_, decomposed) in EXAMINEE {
            let converted = decompose_into_hfs_nfd(*decomposed);
            assert_eq!(&converted, *decomposed);
        }
    }
    #[test]
    fn compose_immutable_fixed_strings_test() {
        for s in EXAMINEE_IMMUTABLE {
            let converted = compose_from_hfs_nfd(*s);
            assert_eq!(&converted, *s);
        }
    }
    #[test]
    fn decompose_immutable_fixed_strings_test() {
        for s in EXAMINEE_IMMUTABLE {
            let converted = decompose_into_hfs_nfd(*s);
            assert_eq!(&converted, *s);
        }
    }

    #[cfg(feature = "bench")]
    fn join_all_materials() -> String {
        EXAMINEE
            .iter()
            .map(|(nfc, nfd)| nfc.to_string() + nfd)
            .collect::<String>()
            + &(EXAMINEE_IMMUTABLE.iter().map(|s| *s).collect::<String>())
    }
    #[cfg(feature = "bench")]
    #[bench]
    fn compose_bench(b: &mut Bencher) {
        let input = join_all_materials();
        b.iter(|| compose_from_hfs_nfd(&input));
    }

    #[cfg(feature = "bench")]
    #[bench]
    fn decompose_bench(b: &mut Bencher) {
        let input = join_all_materials();
        b.iter(|| decompose_into_hfs_nfd(&input));
    }
}
