use std::{cell::RefCell, collections::HashMap};
use web_sys::HtmlAudioElement;

type SongMap = HashMap<String, HtmlAudioElement>;

thread_local! {
    pub static AUDIO: RefCell<SongMap> = RefCell::new(create_song_map());
}

fn create_song_map() -> SongMap {
    let mut songs = HashMap::new();

    songs.insert(
        String::from("song"),
        HtmlAudioElement::new_with_src("../t.mp3").unwrap(),
    );

    songs
}
