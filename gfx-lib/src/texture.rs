pub struct Texture {
    id: u16,
    w: u32,
    h: u32,
    pixels: Vec<u8>,
}

impl Texture {
    pub fn new(id: u16, w: u32, h: u32, pixels: Vec<u8>) -> Texture {
        Texture { id, w, h, pixels }
    }

    pub fn id(&self) -> u16 {
        self.id
    }
}
