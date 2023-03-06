use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::{
    image::Image,
    mono_font::{ascii::FONT_9X15, MonoTextStyle},
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};

use tinybmp::Bmp;

pub const LOGO_OFFSET_X: i32 = 100;
pub const LOGO_OFFSET_Y: i32 = 100;

pub fn clear_text<D>(target: &mut D, offset: Point, color: Rgb565) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb565>,
{
    const TEXT_H: i32 = 20;
    Rectangle::new(
        Point::new(0, 30) + offset - Point::new(0, 15),
        Size::new(130, TEXT_H as u32),
    )
    .into_styled(PrimitiveStyle::with_fill(color))
    .draw(target)?;

    Ok(())
}

pub fn draw_text<D>(target: &mut D, target_text: &str, offset: Point) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb565>,
{
    let text = Text::new(
        target_text,
        Point::new(30, 30) + offset,
        MonoTextStyle::new(&FONT_9X15, Rgb565::BLACK),
    );

    text.draw(target)?;

    Ok(())
}

pub fn draw_logo<D>(target: &mut D) -> Result<Rectangle, D::Error>
where
    D: DrawTarget<Color = Rgb565>,
{
    let bmp: Bmp<Rgb565> = Bmp::from_slice(include_bytes!("../assets/logo.bmp")).unwrap();
    let image = Image::new(&bmp, Point::new(LOGO_OFFSET_X, LOGO_OFFSET_Y));
    image.draw(target)?;

    Ok(image.bounding_box())
}
