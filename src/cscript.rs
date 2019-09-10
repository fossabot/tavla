//! Uses `cscript` on Windows to run a tiny VB script
//! that calls into .NET speech synthesis.
//!
//! Unsupported on other platforms.

pub use crate::child::Speech;
pub use err::Error;

use crate::token::{PauseDuration::*, Token, Tokenizer};
use crate::version::detect_version_with_arg;
use script::script_path;
use std::ffi::OsStr;
use std::io::Write;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};

#[derive(Debug)]
pub struct CScriptVoice {
    script_path: PathBuf,
}

/// A [`Voice`](trait.Voice.html) that works by opening `cscript`
/// running a simple VB script that is first placed into an
/// external file.
impl CScriptVoice {
    pub fn new() -> Result<CScriptVoice, Error> {
        detect_version_with_arg("cscript", None).map_err(Error::cscript_not_installed)?;

        let script_path = script_path()?;

        Ok(CScriptVoice { script_path })
    }

    fn spawn(&self) -> Result<Child, Error> {
        Command::new("cscript")
            .arg("//U")
            .arg(&self.script_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::null()) // Ignore standard output
            .stderr(Stdio::null()) // And error too
            .spawn()
            .map_err(Error::cannot_invoke)
    }

    fn invoke_csript<S : AsRef<str>>(&self, sentence: S, to_file: Option<&OsStr>) -> Result<Speech, Error>
    {
        let xml = format_sapi_xml(sentence.as_ref());

        let mut cscript = self.spawn()?;
        let mut pipe = cscript.stdin.take().ok_or_else(Error::cannot_open_pipe)?;

        if let Some(file) = to_file {
            const WRITE_FILE_UUID : &str = "53377b5f-60a2-4c05-a4eb-55de35452a2b\r\n";
            const NEWLINE : &str = "\r\n";
            write_wide(&mut pipe, OsStr::new(WRITE_FILE_UUID))?;
            write_wide(&mut pipe, file)?;
            write_wide(&mut pipe, OsStr::new(NEWLINE))?;
        }

        write_wide(&mut pipe, OsStr::new(&xml))?;

        pipe.flush().map_err(Error::cannot_write)?;

        Ok(Speech::new(cscript))
    }
}

impl crate::Voice for CScriptVoice {
    type Speech = Speech;
    type Error = Error;

    /// Speaks the given sentence. Emphasized words can be wrapped in underscores.
    fn speak<S>(&self, sentence: S) -> Result<Self::Speech, Self::Error>
    where
        S: AsRef<str>,
    {
        self.invoke_csript(sentence.as_ref(), None)
    }

    fn speak_to_file<S, P>(
        &self,
        sentence: S,
        wav_file_path: P,
    ) -> Result<Self::Speech, Self::Error>
    where
        S: AsRef<str>,
        P: AsRef<Path>,
    {
        self.invoke_csript(sentence.as_ref(), Some(wav_file_path.as_ref().as_os_str()))
    }
}

fn format_sapi_xml(sentence: &str) -> String {
    let mut xml = String::new();

    xml.push_str("<sapi>");
    for token in Tokenizer::new(sentence.as_ref()) {
        match token {
            Token::Normal(text) => xml.push_str(text),
            Token::Emphasised(text) => {
                xml.push_str("<emph>");
                xml.push_str(text);
                xml.push_str("</emph>");
            }
            Token::Pause(Sentence) => {
                xml.push_str("<silence msec=\"350\" />");
            }
            Token::Pause(Paragraph) => {
                xml.push_str("<silence msec=\"700\" />");
            }
            Token::Pause(Seconds(secs)) => {
                xml.push_str(&format!("<silence msec=\"{}000\" />", secs));
            }
        }
    }
    xml.push_str("</sapi>");

    xml
}

fn write_wide(pipe: &mut ChildStdin, data: &OsStr) -> Result<(), Error> {
    for code_point_16 in data.encode_wide() {
        pipe.write(&char_bytes(code_point_16))
            .map_err(Error::cannot_write)?;
    }
    Ok(())
}

fn char_bytes(code_point_16: u16) -> [u8; 2] {
    // NOTE whether this works or not depends on endianness
    let lower_byte = code_point_16 as u8;
    let upper_byte = (code_point_16 >> 8) as u8;
    [lower_byte, upper_byte]
}

mod script {
    use super::Error;
    use std::env::temp_dir;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    const VISUAL_BASIC_SAY_SCRIPT_CONTENTS: &[u8] =
        include_bytes!("../resources/say_lines.vbs");
    // Make sure different tavla versions do not interfere with
    // each other by prepending the crate version to the generated
    // file name.
    const VISUAL_BASIC_SAY_SCRIPT_FILENAME_UNQUALIFIED: &str = "say_lines_for_tavla_.vbs";

    /// If the temporary file directory already contains a file with a name
    /// `VISUAL_BASIC_SAY_SCRIPT_FILENAME`, returns a path to it.
    /// Otherwise creates the file, writes `VISUAL_BASIC_SAY_SCRIPT_CONTENTS`
    /// into it, and then returns the path.
    pub fn script_path() -> Result<PathBuf, Error> {
        let script_path = {
            let mut dir = temp_dir();
            dir.push(format!(
                "{}-{}",
                env!("CARGO_PKG_VERSION"),
                VISUAL_BASIC_SAY_SCRIPT_FILENAME_UNQUALIFIED
            ));
            dir
        };

        if !script_path.exists() {
            File::create(&script_path)
                .and_then(|mut f| f.write_all(VISUAL_BASIC_SAY_SCRIPT_CONTENTS))
                .map_err(Error::cannot_generate_script)?;
        }

        Ok(script_path)
    }
}

mod err {
    use crate::version::Error as VersionDetectError;
    use failure::{Backtrace, Fail};
    use std::io::Error as IoError;

    #[derive(Debug, Fail)]
    pub enum Error {
        #[fail(display = "cscript command could not be found: {}", _0)]
        CscriptNotInstalled(#[cause] VersionDetectError),
        #[fail(
            display = "could not generate speech script for consumption by cscript: {}",
            cause
        )]
        CannotGenerateScript {
            #[cause]
            cause: IoError,
            backtrace: Backtrace,
        },
        #[fail(display = "cscript command could not be invoked: {}", cause)]
        CannotInvoke {
            #[cause]
            cause: IoError,
            backtrace: Backtrace,
        },
        #[fail(display = "cscript command could not be written to: {}", cause)]
        CannotWrite {
            #[cause]
            cause: IoError,
            backtrace: Backtrace,
        },
        #[fail(display = "cannot open pipe to cscript")]
        CannotOpenPipe(Backtrace),
    }

    impl Error {
        pub fn cscript_not_installed(cause: VersionDetectError) -> Self {
            Error::CscriptNotInstalled(cause)
        }

        pub fn cannot_generate_script(cause: IoError) -> Self {
            Error::CannotGenerateScript {
                cause,
                backtrace: Backtrace::new(),
            }
        }

        pub fn cannot_invoke(cause: IoError) -> Self {
            Error::CannotInvoke {
                cause,
                backtrace: Backtrace::new(),
            }
        }

        pub fn cannot_write(cause: IoError) -> Self {
            Error::CannotWrite {
                cause,
                backtrace: Backtrace::new(),
            }
        }

        pub fn cannot_open_pipe() -> Self {
            Error::CannotOpenPipe(Backtrace::new())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::speech::Speech;
    use crate::voice::Voice;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn cscript_to_file() {
        // given
        let say = CScriptVoice::new().unwrap();
        let tempdir = tempdir().expect("could not make temporary directory for test");
        let target_path = tempdir.path().join("testsay.wav");

        // when
        say.speak_to_file("This is a test sentence to speak.", &target_path)
            .expect("Failed to start speaking to file")
            .await_done()
            .expect("Failed to wait until speaking to file is done");

        let generated_file_meta = File::open(&target_path)
            .expect("could not open generated file")
            .metadata()
            .expect("could not obtain metadata of generated file");

        // then
        assert!(
            target_path.exists(),
            "Expecting speaking to path to produce a file."
        );
        assert!(
            generated_file_meta.len() > 1024,
            "Expected test sentence to add up to more than a KiB worth of WAV."
        );
    }
}
