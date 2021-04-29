pub trait Renderer {
    fn draw_sprite(spr:&Sprite);
}

pub trait Sprite {
    fn as_u8_slice(&self) -> &[u8];
}