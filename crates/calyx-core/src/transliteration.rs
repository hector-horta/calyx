

pub fn transliterate(text: &str) -> Option<String> {
    if !text.chars().any(|c| matches!(c, '\u{AC00}'..='\u{D7A3}' | '\u{3041}'..='\u{3096}' | '\u{30A1}'..='\u{30FA}')) {
        return None;
    }

    const CHOSUNG: [&str; 19] = ["g", "kk", "n", "d", "tt", "r", "m", "b", "pp", "s", "ss", "", "j", "jj", "ch", "k", "t", "p", "h"];
    const JUNGSUNG: [&str; 21] = ["a", "ae", "ya", "yae", "eo", "e", "yeo", "ye", "o", "wa", "wae", "oe", "yo", "u", "wo", "we", "wi", "yu", "eu", "ui", "i"];
    const JONGSUNG: [&str; 28] = ["", "k", "k", "ks", "n", "nj", "nh", "t", "l", "lg", "lm", "lb", "ls", "lt", "lp", "lh", "m", "p", "ps", "t", "t", "ng", "t", "t", "k", "t", "p", "h"];


    let mut result = String::new();
    for c in text.chars() {
        if c >= '\u{AC00}' && c <= '\u{D7A3}' {
            let index = (c as u32 - 0xAC00) as usize;
            let chosung_idx = index / (21 * 28);
            let jungsung_idx = (index % (21 * 28)) / 28;
            let jongsung_idx = index % 28;

            if let Some(chosung) = CHOSUNG.get(chosung_idx) {
                result.push_str(chosung);
            }
            if let Some(jungsung) = JUNGSUNG.get(jungsung_idx) {
                result.push_str(jungsung);
            }
            result.push_str(JONGSUNG[jongsung_idx]);
        } else if let Some(romaji) = kana_to_romaji(c) {




            result.push_str(romaji);
        } else {
            result.push(c);
        }
    }

    Some(result)
}

fn kana_to_romaji(c: char) -> Option<&'static str> {
    match c {
        'あ' | 'ア' => Some("a"), 'い' | 'イ' => Some("i"), 'う' | 'ウ' => Some("u"), 'え' | 'エ' => Some("e"), 'お' | 'オ' => Some("o"),
        'か' | 'カ' => Some("ka"), 'き' | 'キ' => Some("ki"), 'く' | 'ク' => Some("ku"), 'け' | 'ケ' => Some("ke"), 'こ' | 'コ' => Some("ko"),
        'さ' | 'サ' => Some("sa"), 'し' | 'シ' => Some("shi"), 'す' | 'ス' => Some("su"), 'せ' | 'セ' => Some("se"), 'そ' | 'ソ' => Some("so"),
        'た' | 'タ' => Some("ta"), 'ち' | 'チ' => Some("chi"), 'つ' | 'ツ' => Some("tsu"), 'て' | 'テ' => Some("te"), 'と' => Some("to"),
        'な' | 'ナ' => Some("na"), 'に' | 'ニ' => Some("ni"), 'ぬ' | 'ヌ' => Some("nu"), 'ね' | 'ネ' => Some("ne"), 'の' | 'ノ' => Some("no"),
        'は' | 'ハ' => Some("ha"), 'ひ' | 'ヒ' => Some("hi"), 'ふ' | 'フ' => Some("fu"), 'へ' | 'ヘ' => Some("he"), 'ほ' | 'ホ' => Some("ho"),
        'ま' | 'マ' => Some("ma"), 'み' | 'ミ' => Some("mi"), 'む' | 'ム' => Some("mu"), 'め' | 'メ' => Some("me"), 'も' | 'モ' => Some("mo"),
        'や' | 'ヤ' => Some("ya"), 'ゆ' | 'ユ' => Some("yu"), 'よ' | 'ヨ' => Some("yo"),
        'ら' | 'ラ' => Some("ra"), 'り' | 'リ' => Some("ri"), 'る' | 'ル' => Some("ru"), 'れ' | 'レ' => Some("re"), 'ろ' | 'ロ' => Some("ro"),
        'わ' | 'ワ' => Some("wa"), 'を' | 'ヲ' => Some("wo"), 'ん' | 'ン' => Some("n"),
        'が' | 'ガ' => Some("ga"), 'ぎ' | 'ギ' => Some("gi"), 'ぐ' | 'グ' => Some("gu"), 'げ' | 'ゲ' => Some("ge"), 'ご' | 'ゴ' => Some("go"),
        'ざ' | 'ザ' => Some("za"), 'じ' | 'ジ' => Some("ji"), 'ず' | 'ズ' => Some("zu"), 'ぜ' | 'ゼ' => Some("ze"), 'ぞ' | 'ゾ' => Some("zo"),
        'だ' | 'ダ' => Some("da"), 'ぢ' | 'ヂ' => Some("ji"), 'づ' | 'ヅ' => Some("zu"), 'で' | 'デ' => Some("de"), 'ど' | 'ド' => Some("do"),
        'ば' | 'バ' => Some("ba"), 'び' | 'ビ' => Some("bi"), 'ぶ' | 'ブ' => Some("bu"), 'べ' | 'ベ' => Some("be"), 'ぼ' | 'ボ' => Some("bo"),
        'ぱ' | 'パ' => Some("pa"), 'ぴ' | 'ピ' => Some("pi"), 'ぷ' | 'プ' => Some("pu"), 'ぺ' | 'ペ' => Some("pe"), 'ぽ' | 'ポ' => Some("po"),
        'ー' => Some("-"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_korean_hangul() {
        assert_eq!(transliterate("안녕하세요"), Some("annyeonghaseyo".to_string()));
    }

    #[test]
    fn test_japanese_kana() {
        assert_eq!(transliterate("あした"), Some("ashita".to_string()));
    }

    #[test]
    fn test_ascii_returns_none() {
        assert_eq!(transliterate("Hello, world!"), None);
    }
}
