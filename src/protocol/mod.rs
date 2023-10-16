//! Protocol backends for the widgets

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use dyn_clone::DynClone;
use image::{DynamicImage, Rgb};
use ratatui::{buffer::Buffer, layout::Rect};

use crate::FontSize;

use super::Resize;

pub mod halfblocks;
pub mod kitty;
pub mod iterm;
#[cfg(feature = "sixel")]
pub mod sixel;

/// A fixed image protocol for the [crate::FixedImage] widget.
pub trait Protocol: Send + Sync {
    fn render(&self, area: Rect, buf: &mut Buffer);
}

/// A resizing image protocol for the [crate::ResizeImage] widget.
pub trait ResizeProtocol: Send + Sync + DynClone {
    fn rect(&self) -> Rect;
    fn render(
        &mut self,
        resize: &Resize,
        background_color: Option<Rgb<u8>>,
        area: Rect,
        buf: &mut Buffer,
    );
    /// This method is optional.
    fn reset(&mut self) {}
}

dyn_clone::clone_trait_object!(ResizeProtocol);

#[derive(Clone)]
/// Image source for [crate::protocol::ResizeProtocol]s
///
/// A `[ResizeProtocol]` needs to resize the ImageSource to its state when the available area
/// changes. A `[Protocol]` only needs it once.
///
/// # Examples
/// ```text
/// use image::{DynamicImage, ImageBuffer, Rgb};
/// use ratatui_image::ImageSource;
///
/// let image: ImageBuffer::from_pixel(300, 200, Rgb::<u8>([255, 0, 0])).into();
/// let source = ImageSource::new(image, "filename.png", (7, 14));
/// assert_eq!((43, 14), (source.rect.width, source.rect.height));
/// ```
///
pub struct ImageSource {
    /// The original image without resizing
    pub image: DynamicImage,
    /// The font size of the terminal
    pub font_size: FontSize,
    /// The area that the [`ImageSource::image`] covers, but not necessarily fills
    pub desired: Rect,
    pub hash: u64,
}

impl ImageSource {
    /// Create a new image source
    pub fn new(image: DynamicImage, font_size: FontSize) -> ImageSource {
        let desired =
            ImageSource::round_pixel_size_to_cells(image.width(), image.height(), font_size);

        let mut state = DefaultHasher::new();
        image.as_bytes().hash(&mut state);
        let hash = state.finish();

        ImageSource {
            image,
            font_size,
            desired,
            hash,
        }
    }
    /// Round an image pixel size to the nearest matching cell size, given a font size.
    fn round_pixel_size_to_cells(
        img_width: u32,
        img_height: u32,
        (char_width, char_height): FontSize,
    ) -> Rect {
        let width = (img_width as f32 / char_width as f32).ceil() as u16;
        let height = (img_height as f32 / char_height as f32).ceil() as u16;
        Rect::new(0, 0, width, height)
    }
}
