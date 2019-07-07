#[derive(Debug, PartialEq, Eq)]
pub enum Token<'a> {
    /// A piece of un-emphasised speech.
    Normal(&'a str),
    /// A piece of emphasised speech.
    Emphasised(&'a str),
    // A pause with specified duration
    Pause(PauseDuration),
}

#[derive(Debug, PartialEq, Eq)]
pub enum PauseDuration {
    Sentence,
    Paragraph,
    Seconds(u32),
}

pub struct Tokenizer<'a> {
    rest: &'a str,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Tokenizer<'a> {
        Tokenizer { rest: source }
    }
}

impl<'a> Tokenizer<'a> {}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let rest = self.rest;
        let mut chars = rest.char_indices();
        let (token, consumed_until) = match chars.next()? {
            (_, '_') => {
                // Pauses and emphasis itself stops emphasis part
                let mut chars = chars.skip_while(|(_, next)| *next != '.' && *next != '_');
                let (terminator_char_idx, terminator_char) =
                    chars.next().unwrap_or_else(|| (rest.len(), '\0'));
                let after_terminator_char_idx = chars
                    .next()
                    .map(|(idx, _)| idx)
                    .unwrap_or_else(|| rest.len());

                // Resume after closing emphasis, if any
                let consumed_until = match terminator_char {
                    '_' => after_terminator_char_idx,
                    _ => terminator_char_idx,
                };

                (
                    Token::Emphasised(&rest[1..terminator_char_idx]),
                    consumed_until,
                )
            }
            (pause_start, '.') => {
                let pause_end = chars
                    .skip_while(|(_, next)| *next == '.')
                    .next()
                    .map(|(idx, _)| idx)
                    .unwrap_or_else(|| rest.len());

                let duration = match pause_end - pause_start {
                    1 => PauseDuration::Sentence,
                    2 => PauseDuration::Paragraph,
                    n => PauseDuration::Seconds((n - 2) as u32),
                };

                (Token::Pause(duration), pause_end)
            }
            (normal_start, _) => {
                let normal_end = chars
                    .skip_while(|(_, next)| *next != '.' && *next != '_')
                    .next()
                    .map(|(idx, _)| idx)
                    .unwrap_or_else(|| rest.len());

                (Token::Normal(&rest[normal_start..normal_end]), normal_end)
            }
        };

        self.rest = &rest[consumed_until..];
        Some(token)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn unemphasised_string() {
        let mut tokenizer = Tokenizer::new("holodrio");
        assert_eq!(
            tokenizer.next().expect("Expected a single token"),
            Token::Normal("holodrio")
        );
        assert_eq!(tokenizer.next(), None);
        assert!(tokenizer.next().is_none());
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn emphasised_string() {
        let mut tokenizer = Tokenizer::new("holodrio _there_");
        assert_eq!(tokenizer.next(), Some(Token::Normal("holodrio ")));
        assert_eq!(tokenizer.next(), Some(Token::Emphasised("there")));
        assert_eq!(tokenizer.next(), None);
        assert!(tokenizer.next().is_none());
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn emphasised_unclosed_string() {
        let mut tokenizer = Tokenizer::new("holodrio _there");
        assert_eq!(tokenizer.next(), Some(Token::Normal("holodrio ")));
        assert_eq!(tokenizer.next(), Some(Token::Emphasised("there")));
        assert_eq!(tokenizer.next(), None);
        assert!(tokenizer.next().is_none());
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn emphasised_unclosed_string_with_pause_after() {
        let mut tokenizer = Tokenizer::new("holodrio _there.");
        assert_eq!(tokenizer.next(), Some(Token::Normal("holodrio ")));
        assert_eq!(tokenizer.next(), Some(Token::Emphasised("there")));
        assert_eq!(
            tokenizer.next(),
            Some(Token::Pause(PauseDuration::Sentence))
        );
        assert_eq!(tokenizer.next(), None);
        assert!(tokenizer.next().is_none());
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn mixed_string() {
        let mut tokenizer = Tokenizer::new("plain ... _emph_. ..");
        assert_eq!(tokenizer.next(), Some(Token::Normal("plain ")));
        assert_eq!(
            tokenizer.next(),
            Some(Token::Pause(PauseDuration::Seconds(1)))
        );
        assert_eq!(tokenizer.next(), Some(Token::Normal(" ")));
        assert_eq!(tokenizer.next(), Some(Token::Emphasised("emph")));
        assert_eq!(
            tokenizer.next(),
            Some(Token::Pause(PauseDuration::Sentence))
        );
        assert_eq!(tokenizer.next(), Some(Token::Normal(" ")));
        assert_eq!(
            tokenizer.next(),
            Some(Token::Pause(PauseDuration::Paragraph))
        );
        assert_eq!(tokenizer.next(), None);
        assert_eq!(tokenizer.next(), None);
    }
}
