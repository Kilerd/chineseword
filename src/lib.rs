use itertools::Itertools;
use phf::{phf_set, Set};
use std::cmp::{max, min};
use std::ops::Add;

static ZH_LEFT_PUNC_SET: Set<char> = phf_set! {'（', '【','《','￥'};
static ZH_RIGHT_PUNC_SET: Set<char> = phf_set! {'，','。','？','！','：','；','）','】','》'};
static ZH_MIDDLE_PUNC_SET: Set<char> = phf_set! {'·','～','—','…'};
static ZH_QUOTE_SET: Set<char> = phf_set! {'“','‘','「','『','”','’','」','』'};

static EN_LEFT_PUNC_SET: Set<char> = phf_set! {'(','[','{','@','#','$'};
static EN_RIGHT_PUNC_SET: Set<char> = phf_set! {',','.','?','!',':',';',')',']','}','%'};
static EN_MIDDLE_PUNC_SET: Set<char> =
    phf_set! {'+','-','*','/','\\','=','<','>','_','^','&','|','~'};
static EN_RIGHT_PUNC_DIGIT_SET: Set<char> = phf_set! {'?','!',';',')',']','}','%'};
static EN_QUOTE_SET: Set<char> = phf_set! {'\'','"','`'};

static REMOVE_SPACE_RULE: [(fn(&char) -> bool, fn(&char) -> bool); 19] = [
    (is_zh_char, is_zh_char),
    (is_zh_char, digit),
    (digit, is_zh_char),
    (is_zh_letter, is_en_letter),
    (is_en_letter, is_zh_letter),
    (is_zh_letter, is_en_right_punc),
    (is_en_left_punc, is_zh_letter),
    (is_zh_punc, is_en_char),
    (is_en_char, is_zh_punc),
    (is_en_letter, is_en_right_punc),
    (is_en_left_punc, is_en_letter),
    (is_en_left_punc, is_en_left_punc),
    (is_en_left_punc, is_en_right_punc),
    (is_en_left_punc, is_en_middle_punc),
    (is_en_right_punc, is_en_right_punc),
    (is_en_middle_punc, is_en_right_punc),
    (is_en_middle_punc, is_en_middle_punc),
    (digit, is_en_right_punc),
    (is_en_left_punc, digit),
];
static ADD_SPACE_RULE: [(fn(&char) -> bool, fn(&char) -> bool); 15] = [
    (is_zh_letter, is_en_left_punc),
    (is_zh_letter, is_en_middle_punc),
    (is_en_right_punc, is_zh_letter),
    (is_en_middle_punc, is_zh_letter),
    (is_en_letter, is_en_left_punc),
    (is_en_letter, is_en_middle_punc),
    (is_en_right_punc, is_en_letter),
    (is_en_middle_punc, is_en_letter),
    (is_en_right_punc, is_en_left_punc),
    (is_en_right_punc, is_en_middle_punc),
    (is_en_middle_punc, is_en_left_punc),
    (digit, is_en_left_punc),
    (digit, is_en_middle_punc),
    (is_en_right_punc_digit, digit),
    (is_en_middle_punc, digit),
];

static MINOR_SPACE_RULE: [(fn(&char) -> bool, fn(&char) -> bool); 4] = [
    (is_zh_letter, is_en_letter),
    (is_en_letter, is_zh_letter),
    (is_zh_letter, digit),
    (digit, is_zh_letter),
];

static GUESS_LANG_WINDOW: usize = 3;

#[derive(Debug, PartialEq)]
enum Lang {
    Zh,
    En,
}

fn correct_space(content: &str) -> String {
    let chars = content.chars().into_iter().collect_vec();
    let mut ret = vec![];
    'outer: for i in 0..chars.len() - 1 {
        let x = chars[i];
        if x == ' ' {
            for (l_rule, r_rule) in REMOVE_SPACE_RULE {
                if l_rule(&chars[i - 1]) && r_rule(&chars[i + 1]) {
                    continue 'outer;
                }
            }
            ret.push(x);
        } else {
            ret.push(x);
            for (l_rule, r_rule) in ADD_SPACE_RULE {
                if l_rule(&x) && r_rule(&chars[i + 1]) {
                    ret.push(' ');
                    break;
                }
            }
        }
    }
    ret.push(chars[chars.len() - 1]);
    ret.into_iter().collect()
}

fn correct_minor_space(content: &str) -> String {
    let chars = content.chars().into_iter().collect_vec();
    let mut ret = vec![];
    for i in 0..chars.len() - 1 {
        let x = chars[i];
        ret.push(x);
        for (l_rule, r_rule) in MINOR_SPACE_RULE {
            if l_rule(&x) && r_rule(&chars[i + 1]) {
                ret.push(' ');
                break;
            }
        }
    }
    ret.push(chars[chars.len() - 1]);
    ret.into_iter().collect()
}

fn correct_punc_zh(content: &str) -> String {
    static END_PUNC_LIST: [(char, char); 6] = [
        ('，', ','),
        ('。', '.'),
        ('？', '?'),
        ('！', '!'),
        ('：', ':'),
        ('；', ';'),
    ];
    static LEFT_BRACKET: Set<char> = phf_set! {'(','（'};
    static RIGHT_BRACKET: Set<char> = phf_set! {')','）'};
    let mut chars = content.chars().into_iter().collect_vec();

    'outer: for i in 0..chars.len() {
        for (zh_end_punc, en_end_punc) in END_PUNC_LIST {
            if chars[i] == en_end_punc && detect_forward(is_zh_char, &chars, i) {
                chars[i] = zh_end_punc;
                continue 'outer;
            }
        }
        if chars[i] == '(' && detect_forward(is_zh_char, &chars, i) {
            chars[i] = '（';
        } else if chars[i] == ')' && detect_backward(is_zh_char, &chars, i) {
            chars[i] = '）';
        }
        if chars[i] == '（' {
            let mut j = i + 1;
            let mut bracket_count = 0;
            let mut ok = false;
            while j < chars.len() {
                if RIGHT_BRACKET.contains(&chars[j]) {
                    if bracket_count == 0 {
                        ok = true;
                        break;
                    } else {
                        bracket_count -= 1;
                    }
                } else if LEFT_BRACKET.contains(&chars[j]) {
                    bracket_count += 1;
                }
                j += 1;
            }
            if ok && chars[j] == ')' {
                chars[j] = '）';
            }
        }
        if chars[i] == '）' {
            let mut j = i - 1;
            let mut bracket_count = 0;
            let mut ok = false;
            while j >= 0 {
                if LEFT_BRACKET.contains(&chars[j]) {
                    if bracket_count == 0 {
                        ok = true;
                        break;
                    } else {
                        bracket_count -= 1;
                    }
                } else if RIGHT_BRACKET.contains(&chars[j]) {
                    bracket_count += 1;
                }

                j -= 1;
            }
            if ok && chars[j] == '(' {
                chars[j] = '（';
            }
        }
    }
    chars.into_iter().join("")
}
fn correct_punc_en(content: &str) -> String {
    static END_PUNC_LIST: [(char, char); 6] = [
        ('，', ','),
        ('。', '.'),
        ('？', '?'),
        ('！', '!'),
        ('：', ':'),
        ('；', ';'),
    ];
    static LEFT_BRACKET: Set<char> = phf_set! {'(','（'};
    static RIGHT_BRACKET: Set<char> = phf_set! {')','）'};
    let mut chars = content.chars().into_iter().collect_vec();

    'outer: for i in 0..chars.len() {
        for (zh_end_punc, en_end_punc) in END_PUNC_LIST {
            if chars[i] == zh_end_punc && detect_forward(is_en_char, &chars, i) {
                chars[i] = en_end_punc;
                continue 'outer;
            }
        }
        if chars[i] == '（' && detect_forward(is_en_char, &chars, i) {
            chars[i] = '(';
        } else if chars[i] == '）' && detect_backward(is_en_char, &chars, i) {
            chars[i] = ')';
        }
        if chars[i] == '(' {
            let mut j = i + 1;
            let mut bracket_count = 0;
            let mut ok = false;
            while j < chars.len() {
                if RIGHT_BRACKET.contains(&chars[j]) {
                    if bracket_count == 0 {
                        ok = true;
                        break;
                    } else {
                        bracket_count -= 1;
                    }
                } else if LEFT_BRACKET.contains(&chars[j]) {
                    bracket_count += 1;
                }
                j += 1;
            }
            if ok && chars[j] == '）' {
                chars[j] = ')';
            }
        }
        if chars[i] == ')' {
            let mut j = i - 1;
            let mut bracket_count = 0;
            let mut ok = false;
            while j >= 0 {
                if LEFT_BRACKET.contains(&chars[j]) {
                    if bracket_count == 0 {
                        ok = true;
                        break;
                    } else {
                        bracket_count -= 1;
                    }
                } else if RIGHT_BRACKET.contains(&chars[j]) {
                    bracket_count += 1;
                }

                j -= 1;
            }
            if ok && chars[j] == '（' {
                chars[j] = '(';
            }
        }
    }
    chars.into_iter().join("")
}

fn correct_quote_zh(content: &str) -> String {
    static DOUBLE_QUOTE_LIST: Set<char> = phf_set! {'"', '“', '”'};
    static SINGLE_QUOTE_LIST: Set<char> = phf_set! {'‘', '’'};
    let mut chars = content.chars().into_iter().collect_vec();
    let mut quote_state = 0;
    let mut quote_state_2 = 0;
    for i in 0..chars.len() {
        if DOUBLE_QUOTE_LIST.contains(&chars[i]) {
            if quote_state == 0 {
                chars[i] = '“';
                quote_state = 1;
            } else {
                chars[i] = '”';
                quote_state = 0;
            }
            if i > 0 && chars[i - 1] == ' ' {
                chars[i - 1] = '\u{0}';
            }
            if i < chars.len() - 1 && chars[i + 1] == ' ' {
                chars[i + 1] = '\u{0}';
            }
        } else if SINGLE_QUOTE_LIST.contains(&chars[i]) {
            if quote_state_2 == 0 {
                chars[i] = '‘';
                quote_state_2 = 1;
            } else {
                chars[i] = '’';
                quote_state_2 = 0;
            }
            if i > 0 && chars[i - 1] == ' ' {
                chars[i - 1] = '\u{0}';
            }
            if i < chars.len() - 1 && chars[i + 1] == ' ' {
                chars[i + 1] = '\u{0}';
            }
        }
    }
    chars.into_iter().filter(|it| it != &'\u{0}').collect()
}
fn correct_quote_en(content: &str) -> String {
    static DOUBLE_QUOTE_LIST: Set<char> = phf_set! {'"', '“', '”'};
    let mut chars = content.chars().into_iter().collect_vec();
    let mut quote_state = 0;
    let mut i = 0;
    while i < chars.len() {
        if DOUBLE_QUOTE_LIST.contains(&chars[i]) {
            if quote_state == 0 {
                quote_state = 1;
                chars[i] = '"'; // todo tex quote
                if i > 0 && chars[i - 1] != ' ' {
                    i += 1;
                    chars.insert(i - 1, ' ');
                }
                if i < chars.len() - 1 && chars[i + 1] == ' ' {
                    chars[i + 1] = '\u{0}';
                }
            } else {
                quote_state = 0;
                chars[i] = '"';
                if i > 0 && chars[i - 1] == ' ' {
                    chars[i - 1] = '\u{0}';
                }
                if i < chars.len() - 1 && chars[i + 1] != ' ' {
                    chars.insert(i, ' ');
                    i += 1;
                }
            }
        } else if chars[i] == '‘' {
            chars[i] = '\''; // todo tex quote
        } else if chars[i] == '’' {
            chars[i] = '\'';
        }
        i += 1;
    }
    chars.into_iter().filter(|it| it != &'\u{0}').collect()
}

fn correct_ellipsis(content: &str, ellipsis: &str) -> String {
    static ELLIPSIS_LIST: Set<char> = phf_set! {'.','。','·','…'};
    let mut chars = content.chars().into_iter().collect_vec();
    let mut i = 0;
    while i < chars.len() {
        if ELLIPSIS_LIST.contains(&chars[i]) {
            let mut ellipsis_count = if chars[i] == '…' { 3 } else { 1 };
            let mut j = i + 1;
            while j < chars.len() {
                if ELLIPSIS_LIST.contains(&chars[j]) {
                    ellipsis_count += if chars[j] == '…' { 3 } else { 1 };
                } else {
                    break;
                }
                j += 1;
            }
            if ellipsis_count >= 3 {
                dbg!(i, j, ellipsis_count);
                let chars1 = ellipsis.chars().collect_vec();
                let ellipsis_lens = chars1.len();
                for (idx, c) in dbg!(chars1).into_iter().enumerate() {
                    if i + idx >= j {
                        chars.insert(j, c);
                    } else if i + idx >= chars.len() {
                        chars.push(c);
                    } else {
                        chars[i + idx] = c;
                    }
                }
                for k in i + ellipsis_lens..j {
                    chars[k] = '\u{0}';
                }
                dbg!(&chars);
            }
            i = j;
        }
        i += 1;
    }
    chars.into_iter().filter(|it| it != &'\u{0}').collect()
}

fn detect_forward(detector: fn(&char) -> bool, slices: &[char], idx: usize) -> bool {
    if idx == 0 {
        return false;
    }
    if detector(&slices[idx - 1]) {
        return true;
    }
    if !slices[idx - 1].is_ascii_whitespace() {
        return false;
    }
    if idx == 1 {
        return false;
    }
    if detector(&slices[idx - 2]) {
        return true;
    }
    false
}

fn detect_backward(detector: fn(&char) -> bool, slices: &[char], idx: usize) -> bool {
    if idx == slices.len() - 1 {
        return false;
    }
    if detector(&slices[idx + 1]) {
        return true;
    }
    if !slices[idx + 1].is_ascii_whitespace() {
        return false;
    }
    if idx == slices.len() - 2 {
        return false;
    }
    if detector(&slices[idx + 2]) {
        return true;
    }
    false
}

pub fn convert_full_width_char(letter: &str) -> &str {
    match letter {
        "０" => "0",
        "１" => "1",
        "２" => "2",
        "３" => "3",
        "４" => "4",
        "５" => "5",
        "６" => "6",
        "７" => "7",
        "８" => "8",
        "９" => "9",
        "Ａ" => "A",
        "Ｂ" => "B",
        "Ｃ" => "C",
        "Ｄ" => "D",
        "Ｅ" => "E",
        "Ｆ" => "F",
        "Ｇ" => "G",
        "Ｈ" => "H",
        "Ｉ" => "I",
        "Ｊ" => "J",
        "Ｋ" => "K",
        "Ｌ" => "L",
        "Ｍ" => "M",
        "Ｎ" => "N",
        "Ｏ" => "O",
        "Ｐ" => "P",
        "Ｑ" => "Q",
        "Ｒ" => "R",
        "Ｓ" => "S",
        "Ｔ" => "T",
        "Ｕ" => "U",
        "Ｖ" => "V",
        "Ｗ" => "W",
        "Ｘ" => "X",
        "Ｙ" => "Y",
        "Ｚ" => "Z",
        "ａ" => "a",
        "ｂ" => "b",
        "ｃ" => "c",
        "ｄ" => "d",
        "ｅ" => "e",
        "ｆ" => "f",
        "ｇ" => "g",
        "ｈ" => "h",
        "ｉ" => "i",
        "ｊ" => "j",
        "ｋ" => "k",
        "ｌ" => "l",
        "ｍ" => "m",
        "ｎ" => "n",
        "ｏ" => "o",
        "ｐ" => "p",
        "ｑ" => "q",
        "ｒ" => "r",
        "ｓ" => "s",
        "ｔ" => "t",
        "ｕ" => "u",
        "ｖ" => "v",
        "ｗ" => "w",
        "ｘ" => "x",
        "ｙ" => "y",
        "ｚ" => "z",
        "－" => "-",
        "／" => "/",
        "．" => ". ",
        "％" => "%",
        "＃" => "#",
        "＠" => "@",
        "＆" => "&",
        "＜" => "<",
        "＞" => ">",
        "［" => "[",
        "］" => "]",
        "｛" => "{",
        "｝" => "}",
        "＼" => "\\",
        "｜" => "|",
        "＋" => "+",
        "＝" => "=",
        "＿" => "_",
        "＾" => "^",
        "｀" => "`",
        "‘‘" => "“",
        "’’" => "”",
        _ => letter,
    }
}

fn guess_lang(content: &str) -> Lang {
    let vec = content.chars().collect_vec();
    let mut i = 0;
    let mut j = vec.len() - 1;
    while i < j && !is_zh_letter(&vec[i]) && !is_en_letter(&vec[i]) {
        i += 1;
    }
    while i < j && !is_zh_letter(&vec[j]) && !is_en_letter(&vec[j]) {
        j -= 1;
    }
    if i >= j {
        return Lang::Zh;
    }
    let mut zh_count = 0;
    let mut en_count = 0;
    for k in i..min(i + GUESS_LANG_WINDOW, j) {
        if is_zh_letter(&vec[k]) {
            zh_count += 1;
        }
        if is_en_letter(&vec[k]) {
            en_count += 1;
        }
    }
    let i1 = j as isize - GUESS_LANG_WINDOW as isize;
    for k in max(i1, i as isize) as usize..j {
        if is_zh_letter(&vec[k]) {
            zh_count += 1;
        }
        if is_en_letter(&vec[k]) {
            en_count += 1;
        }
    }
    if zh_count * 2 >= en_count {
        Lang::Zh
    } else {
        Lang::En
    }
}

pub fn normalize(content: impl Into<String>) -> String {
    let content = content.into();
    let lines = content.lines();
    let mut ret = String::new();
    for line in lines {
        let trimmed = line.trim().split_whitespace().join(" ");

        let mut trimmed = trimmed
            .chars()
            .into_iter()
            .map(|it| convert_full_width_char(&it.to_string()).to_string())
            .join("");
        dbg!(&trimmed.chars());
        let lang = guess_lang(&trimmed);
        loop {
            let last_edit = trimmed.to_string();
            dbg!(&last_edit);
            match lang {
                Lang::Zh => {
                    trimmed = correct_space(&trimmed);
                    dbg!(&trimmed.chars());
                    trimmed = correct_punc_zh(&trimmed);
                    dbg!(&trimmed.chars());
                    trimmed = correct_quote_zh(&trimmed);
                    dbg!(&trimmed.chars());
                    trimmed = correct_ellipsis(&trimmed, "……");
                }
                Lang::En => {
                    trimmed = dbg!(correct_space(&trimmed));
                    dbg!(&trimmed.chars());
                    trimmed = correct_punc_en(&trimmed);
                    trimmed = correct_quote_en(&trimmed);
                    trimmed = correct_ellipsis(&trimmed, "...");
                }
            }
            if last_edit.eq(&trimmed) {
                break;
            }
        }

        trimmed = correct_minor_space(&trimmed);
        dbg!(&trimmed.chars());
        ret.push_str(&trimmed);
    }
    ret
}

fn is_zh_letter(letter: &char) -> bool {
    letter >= &'\u{4e00}' && letter <= &'\u{9fa5}'
}

fn is_zh_left_punc(letter: &char) -> bool {
    ZH_LEFT_PUNC_SET.contains(letter)
}

fn is_zh_right_punc(letter: &char) -> bool {
    ZH_RIGHT_PUNC_SET.contains(letter)
}
fn is_zh_middle_punc(letter: &char) -> bool {
    ZH_MIDDLE_PUNC_SET.contains(letter)
}
fn is_zh_quote(letter: &char) -> bool {
    ZH_QUOTE_SET.contains(letter)
}
fn is_zh_punc(letter: &char) -> bool {
    is_zh_left_punc(letter) || is_zh_middle_punc(letter) || is_zh_right_punc(letter)
}

fn is_zh_char(letter: &char) -> bool {
    is_zh_letter(letter) || is_zh_punc(letter) || is_zh_quote(letter)
}

fn is_en_letter(letter: &char) -> bool {
    letter.is_ascii_alphabetic()
}

fn is_en_left_punc(letter: &char) -> bool {
    EN_LEFT_PUNC_SET.contains(letter)
}
fn is_en_right_punc(letter: &char) -> bool {
    EN_RIGHT_PUNC_SET.contains(letter)
}
fn is_en_right_punc_digit(letter: &char) -> bool {
    EN_RIGHT_PUNC_DIGIT_SET.contains(letter)
}
fn is_en_middle_punc(letter: &char) -> bool {
    EN_MIDDLE_PUNC_SET.contains(letter)
}
fn is_en_quote(letter: &char) -> bool {
    EN_QUOTE_SET.contains(letter)
}
fn is_en_punc(letter: &char) -> bool {
    is_en_middle_punc(letter) || is_en_left_punc(letter) || is_en_right_punc(letter)
}
fn is_en_char(letter: &char) -> bool {
    is_en_letter(letter) || is_en_punc(letter) || is_en_quote(letter)
}
fn letter(letter: &char) -> bool {
    is_zh_letter(letter) || is_en_letter(letter)
}

fn punc(letter: &char) -> bool {
    is_zh_punc(letter) || is_en_punc(letter)
}

fn digit(letter: &char) -> bool {
    letter.is_ascii_digit()
}

#[cfg(test)]
mod tests {
    use crate::{guess_lang, normalize, Lang};

    #[test]
    fn should_keep_the_same_given_only_english_word() {
        assert_eq!("hello world", normalize("hello world"));
    }

    #[test]
    fn should_replace_full_width_character() {
        assert_eq!("hello world", normalize("hello ｗorld"))
    }

    #[test]
    fn should_remove_duplicated_space_given_multiple_space_word() {
        assert_eq!("hello world", normalize("hello  world"));
    }
    #[test]
    fn should_correct_minor_space() {
        assert_eq!("中文 abc", normalize("中文abc"));
        assert_eq!("abc 中文", normalize("abc中文"));
        assert_eq!("中文 123", normalize("中文123"));
        assert_eq!("商品 123 元", normalize("商品123元"));
        assert_eq!("商品 123.00 元", normalize("商品123.00元"));
    }

    #[test]
    fn should_correct_space() {
        assert_eq!("中文中文", normalize("中文 中文"));
        assert_eq!("[[", normalize("[ ["));
    }

    #[test]
    fn should_guess_language() {
        assert_eq!(Lang::Zh, guess_lang("中文"));
        assert_eq!(Lang::Zh, guess_lang("中文12312312"));
        assert_eq!(Lang::En, guess_lang("eng"));
        assert_eq!(Lang::Zh, guess_lang("中文eng"));
    }

    #[test]
    fn should_correct_zh_punc() {
        assert_eq!("中文，中文", normalize("中文,中文"));
        assert_eq!("中文（中文）", normalize("中文(中文)"));
        assert_eq!("他们说：“你好啊”", normalize("他们说:\"你好啊\""));
    }

    #[test]
    fn should_correct_en_punc() {
        assert_eq!("hello, world", normalize("hello，world"));
        assert_eq!("hello, world!", normalize("hello，world！"));
        assert_eq!("hello (world)", normalize("hello（world）"));
        assert_eq!("hello \"world\"", normalize("hello“world”"));
    }

    #[test]
    fn should_correct_ellipsis() {
        assert_eq!("中文……", normalize("中文....。....."));
        assert_eq!("中文……", normalize("中文…"));
        assert_eq!("中文……中文", normalize("中文…中文"));
        assert_eq!("中文……中文……", normalize("中文…...中文..."));
        assert_eq!("English...", normalize("English…"));
        assert_eq!("English...", normalize("English……"));
    }
}
