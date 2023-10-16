//! Iterm protocol implementations.
//! Uses [`sixel-bytes`] to draw image pixels, if the terminal [supports] the [Iterm] protocol.
//! Needs the `sixel` feature.
//!
//! [`sixel-bytes`]: https://github.com/benjajaja/sixel-bytes
//! [supports]: https://arewesixelyet.com
//! [Iterm]: https://en.wikipedia.org/wiki/Iterm
use image::{DynamicImage, Rgb};
use ratatui::{buffer::Buffer, layout::Rect};
use std::{cmp::min, io::Cursor};

use super::{Protocol, ResizeProtocol};
use crate::{ImageSource, Resize, Result};

// Fixed sixel protocol
#[derive(Clone, Default)]
pub struct FixedIterm {
    pub data: String,
    pub rect: Rect,
}

impl FixedIterm {
    pub fn from_source(
        source: &ImageSource,
        resize: Resize,
        background_color: Option<Rgb<u8>>,
        area: Rect,
    ) -> Result<Self> {
        let (img, rect) = resize
            .resize(source, Rect::default(), area, background_color, false)
            .unwrap_or_else(|| (source.image.clone(), source.desired));

        let data = encode(img,rect.width.into(),rect.height.into())?;
        Ok(Self { data, rect })
    }
}

// TODO: change E to sixel_rs::status::Error and map when calling
pub fn encode(img: DynamicImage,width:u64,height:u64) -> Result<String> {
    let mut buffer: Cursor<Vec<u8>> = Cursor::new(Vec::new());
    let _ = img.write_to(&mut buffer, image::ImageFormat::Png)?;
    let builder = iterm2img::from_bytes(buffer.into_inner()).inline(true);
    let data = builder.width(width).height(height).preserve_aspect_ratio(false).build();
    return Ok(data);
}

impl Protocol for FixedIterm {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        render(self.rect, &self.data, area, buf, false)
    }
}

fn render(rect: Rect, data: &str, area: Rect, buf: &mut Buffer, overdraw: bool) {

    buf.get_mut(area.left(), area.top())
        .set_symbol(data);

    // Skip entire area
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            buf.get_mut(x, y).set_skip(true);
        }
    }
    buf.get_mut(area.left(), area.top()).set_skip(false);
}

fn render_area(rect: Rect, area: Rect, overdraw: bool) -> Option<Rect> {
    if overdraw {
        return Some(Rect::new(
            area.x,
            area.y,
            min(rect.width, area.width),
            min(rect.height, area.height),
        ));
    }

    if rect.width > area.width || rect.height > area.height {
        return None;
    }
    Some(Rect::new(area.x, area.y, rect.width, rect.height))
}

#[derive(Clone)]
pub struct ItermState {
    source: ImageSource,
    current: FixedIterm,
    hash: u64,
}

impl ItermState {
    pub fn new(source: ImageSource) -> ItermState {
        ItermState {
            source,
            current: FixedIterm::default(),
            hash: u64::default(),
        }
    }
}

impl ResizeProtocol for ItermState {
    fn rect(&self) -> Rect {
        self.current.rect
    }
    fn render(
        &mut self,
        resize: &Resize,
        background_color: Option<Rgb<u8>>,
        area: Rect,
        buf: &mut Buffer,
    ) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let force = self.source.hash != self.hash;
        if let Some((img, rect)) = resize.resize(
            &self.source,
            self.current.rect,
            area,
            background_color,
            force,
        ) {
            match encode(img,rect.width.into(),rect.height.into()) {
                Ok(data) => {
                    let current = FixedIterm { data, rect };
                    self.current = current;
                    self.hash = self.source.hash;
                }
                Err(_err) => {
                    // TODO: save err in struct and expose in trait?
                }
            }
        }

        render(self.current.rect, &self.current.data, area, buf, true);
    }
}
