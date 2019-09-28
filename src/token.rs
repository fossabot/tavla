#[derive(Debug, PartialEq, Eq)]
pub enum Token<'a> {
    /// A piece of un-emphasised speech.
    Normal(&'a str),
    /// A piece of emphasised speech.
    Emphasised(&'a str),
    // A pause with specified duration
    Pause(PauseDuration),
}

impl<'a> Token<'a> {
    fn new(text: &str, style: Style) -> Token {
        match style {
            Style::Emphasised => Token::Emphasised(text),
            Style::Unemphasised => Token::Normal(text),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum PauseDuration {
    Sentence,
    Paragraph,
    Seconds(u32),
}

impl PauseDuration {
    fn from(period_count: usize, newline_count: usize) -> Option<PauseDuration> {
        if newline_count > 1 && period_count < 2 {
            Some(PauseDuration::Paragraph)
        } else {
            match period_count {
                0 => None,
                1 => Some(PauseDuration::Sentence),
                2 => Some(PauseDuration::Paragraph),
                n => Some(PauseDuration::Seconds((n - 2) as u32)),
            }
        }
    }
}

#[derive(Clone, Copy)]
enum Style {
    // Unemphasised spoken text.
    Unemphasised,
    // Pauses and consecutive newlines.
    Emphasised,
}

impl Style {
    fn flip(&self) -> Style {
        if let Style::Unemphasised = self {
            Style::Emphasised
        } else {
            Style::Unemphasised
        }
    }
}

pub struct Tokenizer<'a> {
    rest: &'a str,
    style: Style,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Tokenizer<'a> {
        Tokenizer {
            rest: source,
            style: Style::Unemphasised,
        }
    }
}

impl<'a> Tokenizer<'a> {
    fn consume_token(&mut self) -> Option<Token<'a>> {
        let leading_pause = self.consume_leading_pause();
        leading_pause.or_else(|| self.consume_text())
    }

    /// Consumes initial dot-pauses and consecutive newlines,
    /// returning an optional pause token.
    ///
    // The index of the first non-whitespace and non-pause character
    /// is returned.
    fn consume_leading_pause(&mut self) -> Option<Token<'a>> {
        let mut pause_count = 0;
        let mut newline_count = 0;

        let first_non_whitespace_idx = self
            .rest
            .char_indices()
            .filter(|(_, c)| {
                match c {
                    '.' => {
                        pause_count += 1;
                        false
                    }
                    '\n' => {
                        newline_count += 1;
                        false // parse on
                    }
                    // stop when encountering non-whitespace and non-pause
                    c => !c.is_whitespace(),
                }
            })
            .map(|(idx, _)| idx)
            .next()
            .unwrap_or_else(|| self.rest.len());

        self.rest = &self.rest[first_non_whitespace_idx..];

        PauseDuration::from(pause_count, newline_count).map(Token::Pause)
    }

    /// Consumes text until encountering a newline or change in
    /// emphasis.
    fn consume_text(&mut self) -> Option<Token<'a>> {
        let chars = self.rest.char_indices();
        let state_change = chars
            .skip_while(|(_, c)| match c {
                '_' | '.' | '\n' => false,
                _ => true,
            })
            .next();

        if let Some((token_end_idx, c)) = state_change {
            if token_end_idx > 0 {
                // first some text, then a pause or emphasis, consume the text
                let text = &self.rest[0..token_end_idx].trim_end();
                self.rest = &self.rest[token_end_idx..];
                Some(Token::new(text, self.style))
            } else if c == '_' {
                // first non-whitespace is emphasis start, consume and recur
                self.style = self.style.flip();
                self.rest = &self.rest[token_end_idx..];
                let next_char_idx = self
                    .rest
                    .char_indices()
                    .skip(1) // skip to first char after emphasis
                    .next()
                    .map(|(idx, _)| idx)
                    .unwrap_or_else(|| self.rest.len());
                self.rest = &self.rest[next_char_idx..];
                self.next()
            } else {
                // starts with some pause construct, this should not be possible
                unreachable!("consume_leading_pause should have consumed leading whitespace")
            }
        } else {
            // text includes the rest of the string
            let remaining_text = &self.rest[0..self.rest.len()];
            let remaining_text = remaining_text.trim_end();
            self.rest = &self.rest[self.rest.len()..];

            if remaining_text.is_empty() {
                None
            } else {
                Some(Token::new(remaining_text, self.style))
            }
        }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rest.is_empty() {
            None
        } else {
            self.consume_token()
        }
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
        let mut tokenizer = Tokenizer::new("   holodrio _there_");
        assert_eq!(tokenizer.next(), Some(Token::Normal("holodrio")));
        assert_eq!(tokenizer.next(), Some(Token::Emphasised("there")));
        assert_eq!(tokenizer.next(), None);
        assert!(tokenizer.next().is_none());
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn emphasised_unclosed_string() {
        let mut tokenizer = Tokenizer::new("holodrio _there");
        assert_eq!(tokenizer.next(), Some(Token::Normal("holodrio")));
        assert_eq!(tokenizer.next(), Some(Token::Emphasised("there")));
        assert_eq!(tokenizer.next(), None);
        assert!(tokenizer.next().is_none());
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn emphasised_unclosed_string_with_pause_after() {
        let mut tokenizer = Tokenizer::new("holodrio _there.");
        assert_eq!(tokenizer.next(), Some(Token::Normal("holodrio")));
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
        let mut tokenizer = Tokenizer::new("plain ..\n\n\n\n. _emph_. plain again ..");
        assert_eq!(tokenizer.next(), Some(Token::Normal("plain")));
        assert_eq!(
            tokenizer.next(),
            Some(Token::Pause(PauseDuration::Seconds(1)))
        );
        assert_eq!(tokenizer.next(), Some(Token::Emphasised("emph")));
        assert_eq!(
            tokenizer.next(),
            Some(Token::Pause(PauseDuration::Sentence))
        );
        assert_eq!(tokenizer.next(), Some(Token::Normal("plain again")));
        assert_eq!(
            tokenizer.next(),
            Some(Token::Pause(PauseDuration::Paragraph))
        );
        assert_eq!(tokenizer.next(), None);
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn speak_paragraphs() {
        // given
        const PARAGRAPHS: &str = "     
         Welcome, you have reached the suicide cell service
      hotline, how may we help you?
      Press _one_ if your machine does not work for you
      as intended and you require technical support

      Press _two_ if you changed your mind and want your
      suicide fee refunded..

      Press _three_ to learn more about the history of
      McKillys Suicide Cells..._";

        // when
        let tokens: Vec<Token> = Tokenizer::new(PARAGRAPHS).collect();

        // then
        assert_eq!(
            tokens,
            vec![
                Token::Normal("Welcome, you have reached the suicide cell service"),
                Token::Normal("hotline, how may we help you?"),
                Token::Normal("Press"),
                Token::Emphasised("one"),
                Token::Normal("if your machine does not work for you"),
                Token::Normal("as intended and you require technical support"),
                Token::Pause(PauseDuration::Paragraph),
                Token::Normal("Press"),
                Token::Emphasised("two"),
                Token::Normal("if you changed your mind and want your"),
                Token::Normal("suicide fee refunded"),
                Token::Pause(PauseDuration::Paragraph),
                Token::Normal("Press"),
                Token::Emphasised("three"),
                Token::Normal("to learn more about the history of"),
                Token::Normal("McKillys Suicide Cells"),
                Token::Pause(PauseDuration::Seconds(1))
            ]
        );
    }
}
