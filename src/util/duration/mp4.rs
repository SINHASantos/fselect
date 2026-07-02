use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::Path;

use crate::util::duration::DurationExtractor;
use crate::util::Duration;

pub struct Mp4DurationExtractor;

impl DurationExtractor for Mp4DurationExtractor {
    fn supports_ext(&self, ext_lowercase: &str) -> bool {
        "mp4" == ext_lowercase
    }

    fn try_read_duration(&self, path: &Path) -> io::Result<Option<Duration>> {
        let fd = File::open(path)?;
        let mut reader = BufReader::new(fd);
        let context = mp4parse::read_mp4(&mut reader)?;
        // The track header duration is expressed in movie timescale units
        // (often 1000, but 600 for QuickTime-origin files).
        let timescale = context
            .timescale
            .map(|ts| ts.0)
            .filter(|&ts| ts > 0)
            .unwrap_or(1000);
        Ok(context
            .tracks
            .iter()
            .find(|track| track.track_type == mp4parse::TrackType::Video)
            .and_then(|track| {
                track.tkhd.as_ref().map(|tkhd| Duration {
                    length: (tkhd.duration / timescale) as usize,
                })
            }))
    }
}

#[cfg(test)]
mod test {
    use super::Mp4DurationExtractor;
    use crate::util::duration::DurationExtractor;
    use crate::util::Duration;
    use crate::PathBuf;
    use std::error::Error;

    #[test]
    fn test_success() -> Result<(), Box<dyn Error>> {
        let path_string =
            std::env::var("CARGO_MANIFEST_DIR")? + "/resources/test/" + "video/rust-logo-blk.mp4";
        let path = PathBuf::from(path_string);
        assert_eq!(
            Mp4DurationExtractor.try_read_duration(&path)?,
            Some(Duration { length: 1 }),
        );
        Ok(())
    }
}
