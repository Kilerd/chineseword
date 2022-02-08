use phf::{Set, phf_set};


static ZH_LEFT_PUNC_SET: Set<char> = phf_set! {'（', '【','《','￥'};
static ZH_RIGHT_PUNC_SET: Set<char> = phf_set! {'，','。','？','！','：','；','）','】','》'};
static ZH_MIDDLE_PUNC_SET: Set<char> = phf_set! {'·','～','—','…'};

pub fn normalize(content: impl Into<String>) -> String {
    content.into()
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

#[cfg(test)]
mod tests {
    use crate::normalize;

    #[test]
    fn should_keep_the_same_given_only_english_word() {
        assert_eq!("hello world", normalize("hello world"));
    }

    #[test]
    fn should_remove_duplicated_space_given_multiple_space_word() {
        assert_eq!("hello world", normalize("hello  world"));
    }
}
