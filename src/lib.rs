use itertools::Itertools;
use phf::{phf_set, Set};
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
    dbg!(&ret);
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

fn guess_lang(content: &str) {
    todo!()
}

pub fn normalize(content: impl Into<String>) -> String {
    let content = content.into();
    let lines = content.lines();
    let mut ret = String::new();
    for line in lines {
        let trimmed = line.trim().split_whitespace().join(" ");

        let trimmed = trimmed
            .chars()
            .into_iter()
            .map(|it| convert_full_width_char(&it.to_string()).to_string())
            .join("");
        dbg!(&trimmed);
        let trimmed = dbg!(correct_space(&trimmed));
        let trimmed = dbg!(correct_minor_space(&trimmed));
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
    use crate::normalize;

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
    }
}
