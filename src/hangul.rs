const SBASE: u32 = 0xAC00;
const LBASE: u32 = 0x1100;
const VBASE: u32 = 0x1161;
const TBASE: u32 = 0x11A7;
const LCOUNT: u32 = 19;
const VCOUNT: u32 = 21;
const TCOUNT: u32 = 28;
const NCOUNT: u32 = 588; // (VCount * TCount)
const SCOUNT: u32 = 11172; // (LCount * NCount)

pub fn is_hangul_precomposed_syllable(ch: char) -> bool {
    return '\u{AC00}' <= ch && ch <= '\u{D7A3}';
}

pub fn is_hangul_conjoinable_jamo(ch: char) -> bool {
    return '\u{1100}' <= ch && ch <= '\u{1112}'
        || '\u{1161}' <= ch && ch <= '\u{1175}'
        || '\u{11A8}' <= ch && ch <= '\u{11C2}';
}

pub fn compose_hangul_jamos(source: &str) -> String {
    if source.is_empty() {
        return "".to_string();
    }
    let mut it = source.chars();
    let mut result = String::new();
    // temporary variable that contains one character before
    let mut last = it.next().unwrap(); // copy first char
    let mut tentative_composed_syllable = None;
    for ch in it {
        // 1. check to see if two current characters are L and V
        if let Some(lindex) = u32::checked_sub(last as u32, LBASE) {
            if lindex < LCOUNT {
                if let Some(vindex) = u32::checked_sub(ch as u32, VBASE) {
                    if vindex < VCOUNT {
                        // make syllable of form LV
                        last = std::char::from_u32(SBASE + (lindex * VCOUNT + vindex) * TCOUNT)
                            .unwrap();
                        tentative_composed_syllable = Some(last); // reset last

                        continue; // discard ch
                    }
                }
            }
        }
        // 2. check to see if two current characters are LV and T
        if let Some(sindex) = u32::checked_sub(last as u32, SBASE) {
            if sindex < SCOUNT && (sindex % TCOUNT) == 0 {
                if let Some(tindex) = u32::checked_sub(ch as u32, TBASE) {
                    if tindex < TCOUNT {
                        // make syllable of form LVT
                        last = std::char::from_u32(last as u32 + tindex).unwrap();
                        // reset last
                        tentative_composed_syllable = Some(last);
                        continue; // discard ch
                    }
                }
            }
        }
        // if neither case was true, just add the character
        if let Some(ch) = tentative_composed_syllable {
            result.push(ch);
            tentative_composed_syllable = None;
        } else {
            result.push(last);
        }
        last = ch;
    }
    if let Some(ch) = tentative_composed_syllable {
        result.push(ch);
    } else {
        result.push(last);
    }
    return result;
}

pub fn decomopse_hangul_syllable(syllable: char) -> String {
    if !is_hangul_precomposed_syllable(syllable) {
        return syllable.to_string();
    }
    let sindex = syllable as u32 - SBASE;
    let mut result = String::new();
    let l = LBASE + sindex / NCOUNT;
    let v = VBASE + (sindex % NCOUNT) / TCOUNT;
    let t = TBASE + sindex % TCOUNT;
    result.push(std::char::from_u32(l).unwrap());
    result.push(std::char::from_u32(v).unwrap());
    if t != TBASE {
        result.push(std::char::from_u32(t).unwrap());
    }
    return result;
}

#[cfg(test)]
mod test {
    use super::*;
    use lazy_static::lazy_static;

    lazy_static! {
        // (NFC, NFD)
        static ref EXAMINEE: Vec<(&'static str, &'static str)> = vec![
            ("김갑환", "김갑환"),
            ("장거한", "장거한"),
            ("최번개", "최번개"),
            ("한주리", "한주리"),
            ("판문점", "판문점"),
            ("남대문시장", "남대문시장"),
            ("서울역", "서울역"),
            ("김치", "김치"),
            ("지짐이", "지짐이"),
            ("국밥", "국밥"),
            ("물냉면", "물냉면"),
            ("비빔밥", "비빔밥"),
            ("삼성전자", "삼성전자"),
        ];
        static ref MIXED_EXAMINEE: Vec<(&'static str, &'static str)> = vec![
            ("《펌프 잇 업》(Pump It Up), 줄여서 펌프, 펌피럽은 안다미로가 개발한 리듬 게임이다.", "《펌프 잇 업》(Pump It Up), 줄여서 펌프, 펌피럽은 안다미로가 개발한 리듬 게임이다."),
            ("태권도 관장. 정의를 중시하며 악을 용서치 않는 무슨 일이든 진지한 성격의 소유자.", "태권도 관장. 정의를 중시하며 악을 용서치 않는 무슨 일이든 진지한 성격의 소유자.")
        ];
    }

    #[test]
    fn hangul_precomposed_test() {
        for (composed, decomposed) in &*EXAMINEE {
            assert!(composed
                .chars()
                .all(|ch| is_hangul_precomposed_syllable(ch)));
            assert!(decomposed
                .chars()
                .all(|ch| !is_hangul_precomposed_syllable(ch)));
        }
    }
    #[test]
    fn hangul_jamo_test() {
        for (composed, decomposed) in &*EXAMINEE {
            assert!(decomposed.chars().all(|ch| is_hangul_conjoinable_jamo(ch)));
            assert!(composed.chars().all(|ch| !is_hangul_conjoinable_jamo(ch)));
        }
    }

    #[test]
    fn hangul_decomposition_test() {
        for (composed, decomposed) in &*EXAMINEE {
            let genrated = composed
                .chars()
                .map(|ch| decomopse_hangul_syllable(ch))
                .collect::<String>();
            assert_eq!(&genrated, decomposed);
        }
        for (composed, decomposed) in &*MIXED_EXAMINEE {
            let genrated = composed
                .chars()
                .map(|ch| decomopse_hangul_syllable(ch))
                .collect::<String>();
            assert_eq!(&genrated, decomposed);
        }
    }

    #[test]
    fn hangul_composition_test() {
        for (composed, decomposed) in &*EXAMINEE {
            assert_eq!(&compose_hangul_jamos(decomposed), composed);
        }
        for (composed, decomposed) in &*MIXED_EXAMINEE {
            assert_eq!(&compose_hangul_jamos(decomposed), composed);
        }
    }
}
