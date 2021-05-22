use std::error::Error;

use buffer::{Buffer, BufferHandler, LineIter};
use piston_window::types::{Color, Matrix2d, Vec2d};
use piston_window::*;
use rusttype::Font;

pub mod buffer;
pub use piston_window::Key;

pub fn run(config: Configuration<'static>, mut handler: impl BufferHandler) -> Result<(), Box<dyn Error>> {
    let mut window: PistonWindow = WindowSettings::new(config.title, (config.width, config.height))
        .exit_on_esc(true)
        .build()?;
    // window.set_lazy(true);

    let mut glyphs = Glyphs::from_font(
        config.font,
        window.create_texture_context(),
        TextureSettings::new(),
    );

    let mut buffer = Buffer::new(config.buffer_size);
    let glyph_size = glyph_size(&mut glyphs, config.font_size)?;

    while let Some(ev) = window.next() {
        if let Some(str) = ev.text_args() {
            handler.on_text(&mut buffer, str);
        }
        if let Some(Button::Keyboard(key)) = ev.press_args() {
            handler.on_key(&mut buffer, key);
        }

        window.draw_2d(&ev, |c, g, dev| {
            clear([0.0, 0.0, 0.0, 1.0], g);
            let mut renderer = TerminalRenderer::new(g, &mut glyphs, glyph_size, c.get_view_size());
            renderer.draw(&buffer, c.transform).expect("Failed to draw");
            glyphs.factory.encoder.flush(dev);
        });
    }
    Ok(())
}

pub struct TerminalRenderer<'a, C, G> {
    graphics: &'a mut G,
    glyphs: &'a mut C,
    glyph_size: Vec2d,
    view_size: Vec2d,
}

impl<'a, C, G> TerminalRenderer<'a, C, G>
where
    C: CharacterCache,
    G: Graphics<Texture = C::Texture>,
{
    pub fn new(graphics: &'a mut G, glyphs: &'a mut C, glyph_size: Vec2d, view_size: Vec2d) -> Self {
        Self {
            graphics,
            glyphs,
            glyph_size,
            view_size,
        }
    }

    pub fn draw(&mut self, buffer: &Buffer, transform: Matrix2d) -> Result<(), C::Error> {
        let text_trans = transform.trans(0., self.glyph_size[1]);
        let max_col = (self.view_size[0] / self.glyph_size[0]) as u32;
        let max_row = (self.view_size[1] / self.glyph_size[1]) as u32;
        let tail = buffer.tail(max_col, max_row);

        for (col, row, glyph) in LineIter::new(tail, max_col) {
            let x = col as f64 * self.glyph_size[0];
            let y = row as f64 * self.glyph_size[1];
            let char_trans = text_trans.trans(x, y);
            if buffer.is_at_cursor(glyph) {
                self.draw_char('|', glyph.foreground, self.glyph_size[1] as u32, char_trans)?;
            } else if glyph.char != '\n' && glyph.char != '\0' {
                rectangle(
                    glyph.background,
                    [0., 0., self.glyph_size[0], self.glyph_size[1]],
                    transform.trans(x, y),
                    self.graphics,
                );
                self.draw_char(
                    glyph.char,
                    glyph.foreground,
                    self.glyph_size[1] as u32,
                    char_trans,
                )?;
            }
        }
        Ok(())
    }

    fn draw_char(
        &mut self,
        ch: char,
        color: Color,
        font_size: u32,
        transform: Matrix2d,
    ) -> Result<(), C::Error> {
        let character = self.glyphs.character(font_size, ch)?;

        let ch_x = character.left();
        let ch_y = character.advance_height() - character.top();

        Image::new_color(color)
            .src_rect([
                character.atlas_offset[0],
                character.atlas_offset[1],
                character.atlas_size[0],
                character.atlas_size[1],
            ])
            .draw(
                character.texture,
                &DrawState::default(),
                transform.trans(ch_x, ch_y),
                self.graphics,
            );
        Ok(())
    }
}

pub struct Configuration<'a> {
    title: &'a str,
    width: u32,
    height: u32,
    buffer_size: usize,
    font: Font<'a>,
    font_size: u32,
}

impl<'a> Default for Configuration<'a> {
    fn default() -> Self {
        Self {
            title: "rterm",
            width: 640,
            height: 320,
            buffer_size: 1000,
            font: Font::try_from_bytes(include_bytes!("../assets/SourceCodePro-Regular.ttf")).unwrap(),
            font_size: 13,
        }
    }
}

fn glyph_size<C: CharacterCache>(glyphs: &mut C, font_size: u32) -> Result<Vec2d, C::Error> {
    let char = glyphs.character(font_size, ' ')?;
    let glyph_w = char.advance_width().ceil();
    let glyph_h = font_size as f64;
    Ok([glyph_w, glyph_h])
}
