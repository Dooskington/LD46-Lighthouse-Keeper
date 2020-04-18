use rodio::Source;
use std::{
    collections::HashMap,
    io::{Cursor, Read},
};

pub fn play(id: AudioAssetId, audio_db: &AudioAssetDb, is_looping: bool) {
    let device = rodio::default_output_device().unwrap();

    if let Some(clip) = audio_db.asset(&id).cloned() {
        let s = rodio::Decoder::new(std::io::BufReader::new(Cursor::new(clip))).unwrap();
        if is_looping {
            rodio::play_raw(&device, s.convert_samples().repeat_infinite());
        } else {
            rodio::play_raw(&device, s.convert_samples());
        }
    } else {
        eprintln!(
            "Failed to play audio file! Audio asset with id {:?} did not exist!",
            id
        );
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum AudioAssetId {
    MusicBackground = 0,
    SfxBallBounce0 = 1,
    SfxBallBounce1 = 2,
    SfxBallWallHit0 = 3,
    SfxBallWallHit1 = 4,
    SfxBrickBreak0 = 5,
    SfxBrickBreak1 = 6,
    SfxBallDeath0 = 7,
}

pub struct AudioAssetDb {
    assets: HashMap<AudioAssetId, Vec<u8>>,
}

impl AudioAssetDb {
    pub fn new() -> Self {
        AudioAssetDb {
            assets: HashMap::new(),
        }
    }

    pub fn import(&mut self, id: AudioAssetId, path: &str) -> std::io::Result<()> {
        let mut file = std::fs::File::open(path).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        self.assets.insert(id, buffer);

        Ok(())
    }

    pub fn asset(&self, id: &AudioAssetId) -> Option<&Vec<u8>> {
        self.assets.get(id)
    }
}
