use super::code_table::*;
use super::reverse_tree::ReverseTreeNode;
use std::collections::VecDeque;

#[cfg(test)]
mod test {
    use super::*;

    /// Decomposition test on one specific pair
    /// # Arguments
    /// * `src` - A character to be decomposed
    /// * `target` - The target of decomposition
    /// # Panics
    /// When test is failed
    fn try_decompose(src: char, target: &str) {
        assert_eq!(*(MAP_TO_HFS.get(&src).unwrap()), target);
    }
    /// Composition test on one specific pair
    /// # Arguments
    /// * `src` - A string to be composed
    /// * `target` - The target of composition
    /// # Panics
    /// When test is failed
    fn try_compose(src: &str, target: char) {
        let mut sub_dic = &*MAP_TO_NORMAL;
        for element in src.chars().take(src.chars().count() - 1) {
            match sub_dic.get(&element) {
                Some(ReverseTreeNode {
                    current: _,
                    next: Some(next),
                }) => {
                    sub_dic = next;
                }
                Some(_) => {
                    panic!("No next sub-dictionary for U+{:04X} (target: `{}` aka U+{:04X}) in the entry in reverse dictionary",
                        element as u32,
                        target,  target as u32
                    )
                }
                None => panic!(
                    "No entry for U+{:04X} (target: `{}` aka U+{:04X}) in the reverse dictionary",
                    element as u32, target, target as u32
                ),
            }
        }
        let last_element = src.chars().last().unwrap();
        let generated = sub_dic
            .get(&last_element)
            .expect(
                format!(
                    "No entry for U+{:04X} (target: `{}` aka U+{:04X}) in the reverse dictionary",
                    last_element as u32, target, target as u32
                )
                .as_str(),
            )
            .current
            .expect(
                format!(
            "No next sub-dictionary for U+{:04x} (target: `{}` aka U+{:04X}) in the entry in reverse dictionary",
            last_element as u32,
            target, target as u32
        )
                .as_str(),
            );
        assert_eq!(
            generated, target,
            "Composed character different: source: `{}` / target: `{}` (U+{:04X}) / actual: `{}` (U+{:04X})",
            src,
            target, target as u32, generated, generated as u32
        );
    }
    #[test]
    fn normal_to_hfs_fixed() {
        let examinee = vec![
            ('À', "À"),
            ('é', "é"),
            ('ポ', "ポ"),
            ('ᾖ', "\u{03B7}\u{0345}\u{0313}\u{0342}"),
        ];
        for (composed, decomposed) in examinee {
            try_decompose(composed, decomposed);
        }
    }
    #[test]
    fn hfs_to_normal_fixed() {
        let examinee = vec![
            ("ホ\u{309A}", 'ポ'),
            ("\u{03B7}\u{0345}\u{0313}\u{0342}", 'ᾖ'),
            ("\u{0391}\u{0345}", '\u{1FBC}'),
            ("\u{0391}\u{0345}\u{0313}", '\u{1F88}'),
            ("\u{0391}\u{0345}\u{0313}\u{0300}", '\u{1F8A}'),
            ("\u{0391}\u{0345}\u{0313}\u{0301}", '\u{1F8C}'),
        ];
        for (elements, target_composed) in examinee {
            try_compose(elements, target_composed);
        }
    }
    #[test]
    fn hfs_to_normal_all() {
        for (composed, target) in (*MAP_TO_HFS).iter() {
            try_decompose(*composed, *target);
        }
    }

    #[test]
    fn normal_to_hfs_all() {
        let mut queue = VecDeque::new();
        for (ch_ref, node_ref) in &*MAP_TO_NORMAL {
            queue.push_back((ch_ref.to_string(), node_ref));
        }
        while let Some((decomposed_str, node_ref)) = queue.pop_back() {
            if let Some(target) = node_ref.current {
                try_compose(&decomposed_str, target);
            }
            if let Some(next_dic) = &node_ref.next {
                for (ch_ref, node_ref) in next_dic.as_ref() {
                    let mut s = decomposed_str.clone();
                    s.push(*ch_ref);
                    queue.push_back((s, node_ref));
                }
            }
        }
    }
}
