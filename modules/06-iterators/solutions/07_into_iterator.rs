//! # Making your collection work with `for` (reference solution)
//!
//! - The three impls mirror `Vec`'s: `T` yields owned items, `&'a T`
//!   yields `&'a Item`, `&'a mut T` yields `&'a mut Item`. Delegate to the
//!   inner collection's iterators rather than writing your own.
//! - With the `&` impl, `&playlist` works anywhere an `IntoIterator` is
//!   accepted: `for`, `zip`, `extend`, `chain`, or a function like `longest`.
//! - `impl Iterator` hides everything but `Iterator`. Returning the concrete
//!   `std::slice::Iter<'_, Track>` keeps `rev`, `len` and `clone` available
//!   (so would `impl DoubleEndedIterator<Item = &Track> + ExactSizeIterator`).
//! - Convention: `iter()` and `iter_mut()` methods next to the `&` and
//!   `&mut` impls, which then just call them.
//! - In edition 2024, return-position `impl Trait` captures every lifetime
//!   in scope, so `fn durations(&self) -> impl Iterator<Item = u32>` can
//!   borrow `self` without the `+ '_` that edition 2021 needed.

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
    fn iter(&self) -> std::slice::Iter<'_, Track> {
        self.tracks.iter()
    }

    fn iter_mut(&mut self) -> std::slice::IterMut<'_, Track> {
        self.tracks.iter_mut()
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

impl<'a> IntoIterator for &'a Playlist {
    type Item = &'a Track;
    type IntoIter = std::slice::Iter<'a, Track>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut Playlist {
    type Item = &'a mut Track;
    type IntoIter = std::slice::IterMut<'a, Track>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
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
