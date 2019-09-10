extern crate tavla;
#[cfg(target_os = "macos")]
use tavla::{say, Speech, Voice};

#[cfg(target_os = "macos")]
#[test]
fn speak_say() {
    match say() {
        Err(err) => {
            // not being available is an ok outcome, test successful
            println!("say not available: {:?}.", err);
        }
        // If it is, it must be invokable successfully
        Ok(say) => {
            say.speak("Hello with say.... And hello again after a long _pause_.")
                .expect("say obtained, but failed to speak a phrase")
                .await_done()
                .expect("say obtained, but failed to speak a phrase until done");

            println!("say available: {:?}", say);
        }
    }
}
