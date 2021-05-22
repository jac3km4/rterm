use std::iter;

use piston_window::types::Color;
use piston_window::*;

#[derive(Debug)]
pub struct Buffer {
    pub cursor: u32,
    pub glyphs: Vec<Glyph>,
}

impl Buffer {
    pub fn is_at_cursor(&self, glyph: &Glyph) -> bool {
        let item_ref: *const Glyph = &self.glyphs[self.cursor as usize];
        item_ref == glyph
    }
}

impl Buffer {
    pub fn new(size: usize) -> Self {
        let glyphs = iter::repeat(Glyph::new('\0')).take(size).collect();
        Self { cursor: 0, glyphs }
    }

    pub fn push_glyph(&mut self, glyph: Glyph) {
        self.glyphs[self.cursor as usize] = glyph;
        self.seek_cursor(1);
    }

    pub fn push_text(&mut self, text: &str) {
        for char in text.chars() {
            self.push_glyph(Glyph::new(char));
        }
    }

    pub fn seek_cursor(&mut self, n: i32) {
        self.cursor = (self.cursor as i32 + n) as u32 % self.glyphs.len() as u32;
    }

    pub fn tail(&self, max_col: u32, max_row: u32) -> impl ExactSizeIterator<Item = &Glyph> {
        let offset = self.cursor as usize + 1;
        let prefix = self.glyphs.iter().take(offset);
        let postfix = self
            .glyphs
            .iter()
            .skip(offset)
            .take_while(|glyph| glyph.char != '\n' && glyph.char != '\0')
            .collect::<Vec<_>>()
            .into_iter();

        LineIter::new(prefix.chain(postfix).rev(), max_col)
            .take_while(|(_, row, glyph)| {
                let effective_row = if glyph.char == '\n' { *row + 1 } else { *row };
                effective_row < max_row
            })
            .map(|(_, _, glyph)| glyph)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
    }
}

#[derive(Debug, Clone)]
pub struct Glyph {
    pub char: char,
    pub foreground: Color,
    pub background: Color,
}

impl Glyph {
    pub fn new(char: char) -> Self {
        Self {
            char,
            foreground: [1.0, 1.0, 1.0, 1.0],
            background: [0.0, 0.0, 0.0, 0.0],
        }
    }
}

pub trait BufferHandler {
    fn on_key(&mut self, buffer: &mut Buffer, key: Key);
    fn on_text(&mut self, buffer: &mut Buffer, text: String);
}

#[derive(Debug)]
pub struct DefaultHandler;

impl BufferHandler for DefaultHandler {
    fn on_key(&mut self, buffer: &mut Buffer, key: Key) {
        match key {
            Key::Return => buffer.push_text("\n"),
            Key::Left => buffer.seek_cursor(-1),
            Key::Right => buffer.seek_cursor(1),
            Key::Backspace => {
                buffer.seek_cursor(-1);
                buffer.glyphs[buffer.cursor as usize] = Glyph::new(' ');
            }
            _ => (),
        }
    }

    fn on_text(&mut self, buffer: &mut Buffer, text: String) {
        buffer.push_text(&text)
    }
}

#[derive(Debug)]
pub struct LineIter<I> {
    glyphs: I,
    col: u32,
    row: u32,
    max_col: u32,
}

impl<I> LineIter<I> {
    pub fn new(glyphs: I, max_col: u32) -> Self {
        Self {
            glyphs,
            col: 0,
            row: 0,
            max_col,
        }
    }
}

impl<'a, I: Iterator<Item = &'a Glyph>> Iterator for LineIter<I> {
    type Item = (u32, u32, &'a Glyph);

    fn next(&mut self) -> Option<Self::Item> {
        let glyph = self.glyphs.next()?;
        let res = (self.col, self.row, glyph);
        match glyph.char {
            '\n' => {
                self.row += 1;
                self.col = 0;
            }
            _ => {
                self.col += 1;
            }
        }
        if self.col >= self.max_col {
            self.row += 1;
            self.col = 0;
        }
        Some(res)
    }
}
