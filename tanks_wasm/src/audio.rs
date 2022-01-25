use std::{cell::RefCell, collections::HashMap};
use web_sys::HtmlAudioElement;

thread_local! {
    pub static AUDIO: RefCell<SongMap> = RefCell::new(create_song_map());
}

pub type SongMap = HashMap<String, HtmlAudioElement>;
fn create_song_map() -> SongMap {
    let mut songs = HashMap::new();

    songs.insert(
        String::from("song"),
        HtmlAudioElement::new_with_src("../t.mp3").unwrap(),
    );

    songs
}
