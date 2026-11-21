//! # Making your collection work with `for`
//!
//! `for x in thing` calls `IntoIterator::into_iter(thing)`. A collection
//! usually implements it three times: for `Playlist` (consumes it, yields
//! `Track`), for `&Playlist` (yields `&Track`) and for `&mut Playlist`
//! (yields `&mut Track`). Only the first exists so far, and the tests use
//! all three. They also call `.rev()` and `.len()` on `iter()`, which its
//! `impl Iterator` return type doesn't promise. Make it all compile.

#[derive(Debug, Clone, PartialEq)]
struct Track {
    title: String,
    secs: u32,
    plays: u32,
}

#[derive(Debug, Default)]
struct Playlist {
    tracks: Vec<Track>,
}

impl Playlist {
    fn add(&mut self, title: &str, secs: u32) {
        self.tracks.push(Track {
            title: title.to_string(),
            secs,
            plays: 0,
        });
    }

    /// The tracks, in order.
    fn iter(&self) -> impl Iterator<Item = &Track> {
        self.tracks.iter()
    }

    /// Total running time in seconds.
    fn total_secs(&self) -> u32 {
        self.iter().map(|t| t.secs).sum()
    }
}

impl IntoIterator for Playlist {
    type Item = Track;
    type IntoIter = std::vec::IntoIter<Track>;

    fn into_iter(self) -> Self::IntoIter {
        self.tracks.into_iter()
    }
}

/// The longest of any tracks you can iterate by reference: a playlist, a
/// slice, a filtered iterator... The first one on ties.
fn longest<'a>(tracks: impl IntoIterator<Item = &'a Track>) -> Option<&'a Track> {
    tracks
        .into_iter()
        .reduce(|best, t| if t.secs > best.secs { t } else { best })
}

fn main() {
    let mut playlist = Playlist::default();
    playlist.add("Intro", 95);
    playlist.add("Long Drive", 412);
    playlist.add("Outro", 130);

    for track in &mut playlist {
        track.plays += 1;
    }
    for track in &playlist {
        println!("{} ({}s, played {}x)", track.title, track.secs, track.plays);
    }
    println!("longest: {:?}", longest(&playlist).map(|t| &t.title));
    println!("last: {:?}", playlist.iter().next_back().map(|t| &t.title));
    let titles: Vec<String> = playlist.into_iter().map(|t| t.title).collect();
    println!("{titles:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Playlist {
        let mut playlist = Playlist::default();
        playlist.add("Intro", 95);
        playlist.add("Long Drive", 412);
        playlist.add("Outro", 130);
        playlist
    }

    #[test]
    fn loops_over_references() {
        let playlist = sample();
        let mut total = 0;
        for track in &playlist {
            total += track.secs;
        }
        assert_eq!(total, playlist.total_secs());
    }

    #[test]
    fn loops_over_mutable_references() {
        let mut playlist = sample();
        for track in &mut playlist {
            track.plays += 1;
        }
        for track in &mut playlist {
            track.plays *= 10;
        }
        assert!(playlist.iter().all(|t| t.plays == 10));
    }

    #[test]
    fn consumes_the_playlist() {
        let mut titles = Vec::new();
        for track in sample() {
            titles.push(track.title);
        }
        assert_eq!(titles, ["Intro", "Long Drive", "Outro"]);
    }

    #[test]
    fn iter_runs_backwards_and_knows_its_length() {
        let playlist = sample();
        assert_eq!(playlist.iter().len(), 3);
        let last = playlist.iter().rev().next().map(|t| t.title.as_str());
        assert_eq!(last, Some("Outro"));
    }

    #[test]
    fn works_wherever_into_iterator_does() {
        let playlist = sample();
        assert_eq!(longest(&playlist).map(|t| t.secs), Some(412));
        assert_eq!(
            longest(playlist.iter().filter(|t| t.secs < 200)).map(|t| t.secs),
            Some(130)
        );
        let pairs: Vec<(u32, u32)> = playlist
            .iter()
            .zip(&playlist)
            .map(|(a, b)| (a.secs, b.secs))
            .collect();
        assert_eq!(pairs, [(95, 95), (412, 412), (130, 130)]);
        assert_eq!(longest(&Playlist::default()), None);
    }
}
