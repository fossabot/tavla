# tavla
Hey there! _tavla_ is a small rust library that provides a
minimal set of speech synthesis operations on Windows, Mac
and any system with _espeak_ installed.

## Installation
Use your package manager to install `espeak` on the machine
where _tavla_ will run, e.g. on Arch Linux:

    pacman -S espeak

For Mac or Windows, no special setup is required.

## How to use
_tavla_ speaks strings out loud, optionally emphasising words
with underscores, and adding pauses with one or more `.`:

    extern crate tavla;

    fn main() {
        let voice = tavla::any_voice()
            .expect("Could not find espeak or say");

        voice.say("Oh _my_, the computer is _talking_... Neat!")
            .expect("Error occurred while speaking");
    }

## Limitations, Future Plans
_tavla_ was designed to be super easy to set up, but it will
probably not make you happy if you need any of the following:
* low latency (spawning a shell takes some time),
* phoneme output,
* language/voice selection,
* support for other systems than Windows, Mac and those with `espeak` installed,
* future-based async.

There are no current plans to make _tavla_ work with futures,
but it may happen once std futures are more stable.

## Alternatives
If you are doing serious speech synthesis consider using
something more sophisticated like
[speech-dispatcher](https://crates.io/crates/speech-dispatcher)
or linking against espeak directly with
[espeak-sys](https://crates.io/crates/espeak-sys).

