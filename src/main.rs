extern crate dot_vox;
extern crate image;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;
#[macro_use]
extern crate error_chain;

use structopt::StructOpt;
use image::{ImageBuffer, Rgba, RgbaImage};
use dot_vox::{DotVoxData, Voxel, Model};

mod enums;
use enums::{Side, View};

use std::path::PathBuf;
use std::fs::create_dir_all;

#[derive(StructOpt)]
struct Options {
    #[structopt(help = "Input .vox model")]
    filename: String,
    #[structopt(short = "m", long = "model", help = "Which model in the voxel file to render [default: all]")]
    model: Option<usize>,
    #[structopt(short = "s", long = "side", help = "Which side of the model to render [default: all]")]
    side: Option<Side>,
    #[structopt(short = "v", long = "view", help = "Which perspective of the model to render [default: all]")]
    view: Option<View>,
    #[structopt(short = "o", long = "output", default_value = ".", help = "The output directory to write files to")]
    output: String
}

error_chain!{
    foreign_links {
        Io(::std::io::Error);
    }
}

fn run() -> Result<()> {
    let options = Options::from_args();

    Renderer::new(&options.filename)
        .chain_err(|| format!("Failed to parse '{}'.", &options.filename))?
        .render_all(options)
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{}", error);

        for cause in error.iter().skip(1) {
            eprintln!("Caused by: {}", cause);
        }
    }
}

struct Renderer {
    vox: DotVoxData
}

impl Renderer {
    fn new(filename: &str) -> Result<Self> {
        Ok(Self {
            vox: dot_vox::load(filename)?
        })
    }

    fn render_all(&mut self, options: Options) -> Result<()> {
        let models = options.model.map(|model| vec![model]).unwrap_or_else(|| (0 .. self.vox.models.len()).collect());
        let sides  = options.side.map(|side| vec![side]).unwrap_or_else(Side::all);
        let views  = options.view.map(|view| vec![view]).unwrap_or_else(View::all);
        
        for model in models {
            for side in &sides {
                for view in &views {
                    if *view == View::Face || (*side != Side::Top && *side != Side::Bottom) {
                        self.render(model, side, view, PathBuf::from(&options.output))?;
                    }
                }
            }
        }

        Ok(())
    }

    fn render(&mut self, model: usize, side: &Side, view: &View, mut output: PathBuf) -> Result<()> {
        let image = match *view {
            View::Face                  => ModelRenderer::new(self, model, 0, 0).render_face(side),
            View::FourtyFive            => ModelRenderer::new(self, model, 0, 1).render_45(side),
            View::FourtyFiveIso         => ModelRenderer::new(self, model, 1, 1).render_45_iso(side),
            View::TwentyTwoPointFive    => ModelRenderer::new(self, model, 1, 2).render_22_5(side),
            View::TwentyTwoPointFiveIso => ModelRenderer::new(self, model, 3, 3).render_22_5_iso(side),
        };

        if !output.exists() {
            create_dir_all(&output).chain_err(|| format!("Failed to create directory '{}'.", output.display()))?;
        }

        output.push(format!("{}_{}_{}.png", side.to_str(), view.to_str(), model));
        image.save(&output).chain_err(|| format!("Failed to save '{}'.", output.display()))?;
        Ok(())
    }
}

struct Size {
    x: u32,
    y: u32,
    z: u32
}

impl Size {
    fn invert_x(&self, voxel: &Voxel) -> u32 {
        self.x - u32::from(voxel.x)
    }

    fn invert_y(&self, voxel: &Voxel) -> u32 {
        self.y - u32::from(voxel.y)
    }

    fn invert_z(&self, voxel: &Voxel) -> u32 {
        self.z - u32::from(voxel.z)
    }
}

struct ModelRenderer<'a> {
    model: &'a mut Model,
    palette: &'a [u32],
    x_padding: u32,
    y_padding: u32
}

impl<'a> ModelRenderer<'a> {
    fn new(renderer: &'a mut Renderer, model: usize, x_padding: u32, y_padding: u32) -> Self {
        Self {
            model: &mut renderer.vox.models[model],
            palette: &renderer.vox.palette,
            x_padding, y_padding
        }
    }

    fn colour(&self, index: u8, subtract: u8) -> Rgba<u8> {
        let colour = self.palette[index as usize - 1];
        let r = (colour % 256) as u8;
        let g = ((colour >> 8)  % 256) as u8;
        let b = ((colour >> 16) % 256) as u8;
        let a = ((colour >> 24) % 256) as u8;

        let r = r.saturating_sub(subtract);
        let g = g.saturating_sub(subtract);
        let b = b.saturating_sub(subtract);

        Rgba {
            data: [r, g, b, a]
        }
    }

    fn create_image<S, X, Y>(&mut self, sort: S, map_x: X, map_y: Y) -> RgbaImage
        where
            S: Fn(&Voxel) -> u32,
            X: Fn(&Voxel) -> u32,
            Y: Fn(&Voxel) -> u32
    {
        self.model.voxels.sort_unstable_by_key(sort);
        
        let width  = self.model.voxels.iter().map(map_x).max().unwrap_or(0) + self.x_padding + 1;
        let height = self.model.voxels.iter().map(map_y).max().unwrap_or(0) + self.y_padding + 1;
        
        ImageBuffer::new(width, height)
    }

    fn size(&self) -> Size {
        Size {
            x: self.model.size.x,
            y: self.model.size.y,
            z: self.model.size.z,
        }
    }

    fn render_face(&mut self, side: &Side) -> RgbaImage {
        let size = self.size();

        match *side {
            Side::Top => self.render_face_closure(
                |voxel| u32::from(voxel.z), |voxel| u32::from(voxel.x), |voxel| size.invert_y(voxel),
            ),
            Side::Front  => self.render_face_closure(
                |voxel| size.invert_y(voxel), |voxel| u32::from(voxel.x), |voxel| size.invert_z(voxel)
            ),
            Side::Left => self.render_face_closure(
                |voxel| size.invert_x(voxel), |voxel| size.invert_y(voxel), |voxel| size.invert_z(voxel)
            ),
            Side::Right => self.render_face_closure(
                |voxel| u32::from(voxel.x), |voxel| u32::from(voxel.y), |voxel| size.invert_z(voxel)
            ),
            Side::Back => self.render_face_closure(
                |voxel| u32::from(voxel.y), |voxel| size.invert_x(voxel), |voxel| size.invert_z(voxel)
            ),
            Side::Bottom => self.render_face_closure(
                |voxel| size.invert_z(voxel), |voxel| u32::from(voxel.x), |voxel| u32::from(voxel.y)
            )
        }
    }

    fn render_face_closure<S, X, Y>(&mut self, sort: S, map_x: X, map_y: Y) -> RgbaImage
        where
            S: Fn(&Voxel) -> u32,
            X: Fn(&Voxel) -> u32,
            Y: Fn(&Voxel) -> u32
    {
        let mut image = self.create_image(sort, &map_x, &map_y);

        for voxel in &self.model.voxels {
            let colour = self.colour(voxel.i, 0);
            image.put_pixel(map_x(voxel), map_y(voxel), colour);
        }

        image
    }

    fn render_45(&mut self, side: &Side) -> RgbaImage {
        let size = self.size();

        match *side {
            Side::Front => self.render_45_closure(
                |voxel| u32::from(voxel.z) + size.invert_y(voxel),
                |voxel| u32::from(voxel.x),
                |voxel| size.invert_z(voxel) + size.invert_y(voxel)
            ),
            Side::Left =>  self.render_45_closure(
                |voxel| u32::from(voxel.z) + size.invert_x(voxel),
                |voxel| size.invert_y(voxel),
                |voxel| size.invert_z(voxel) + size.invert_x(voxel)
            ),
            Side::Right => self.render_45_closure(
                |voxel| u32::from(voxel.z) + u32::from(voxel.x),
                |voxel| u32::from(voxel.y),
                |voxel| size.invert_z(voxel) + u32::from(voxel.x)
            ),
            Side::Back  => self.render_45_closure(
                |voxel| u32::from(voxel.z) + u32::from(voxel.y),
                |voxel| size.invert_x(voxel),
                |voxel| size.invert_z(voxel) + u32::from(voxel.y)
            ),
            _ => unreachable!()
        }
    }


    fn render_45_closure<S, X, Y>(&mut self, sort: S, map_x: X, map_y: Y) -> RgbaImage
        where
            S: Fn(&Voxel) -> u32,
            X: Fn(&Voxel) -> u32,
            Y: Fn(&Voxel) -> u32
    {
        let mut image = self.create_image(sort, &map_x, &map_y);

        for voxel in &self.model.voxels {
            let x = map_x(voxel);
            let y = map_y(voxel);

            let colour = self.colour(voxel.i, 30);
            let colour_lighter = self.colour(voxel.i, 0);

            image.put_pixel(x, y + 1, colour);
            image.put_pixel(x, y, colour_lighter);
        }

        image
    }

    fn render_22_5(&mut self, side: &Side) -> RgbaImage {
        let size = self.size();
        
        match *side {
            Side::Front => self.render_22_5_closure(
                |voxel| u32::from(voxel.z) + size.invert_y(voxel),
                |voxel| u32::from(voxel.x) * 2,
                |voxel| size.invert_z(voxel) * 2 + size.invert_y(voxel)
            ),
            Side::Left => self.render_22_5_closure(
                |voxel| u32::from(voxel.z) + size.invert_x(voxel),
                |voxel| size.invert_y(voxel) * 2,
                |voxel| size.invert_z(voxel) * 2 + size.invert_x(voxel)
            ),
            Side::Right => self.render_22_5_closure(
                |voxel| u32::from(voxel.z) + u32::from(voxel.x),
                |voxel| u32::from(voxel.y) * 2,
                |voxel| size.invert_z(voxel) * 2 + u32::from(voxel.x)
            ),
            Side::Back  => self.render_22_5_closure(
                |voxel| u32::from(voxel.z) + u32::from(voxel.y),
                |voxel| size.invert_x(voxel) * 2,
                |voxel| size.invert_z(voxel) * 2 + u32::from(voxel.y)
            ),
            _ => unreachable!()
        }
    }


    fn render_22_5_closure<S, X, Y>(&mut self, sort: S, map_x: X, map_y: Y) -> RgbaImage
        where
            S: Fn(&Voxel) -> u32,
            X: Fn(&Voxel) -> u32,
            Y: Fn(&Voxel) -> u32
    {
        let mut image = self.create_image(sort, &map_x, &map_y);

        for voxel in &self.model.voxels {
            let x = map_x(voxel);
            let y = map_y(voxel);

            let colour = self.colour(voxel.i, 30);
            let colour_lighter = self.colour(voxel.i, 0);

            image.put_pixel(x,     y,     colour_lighter);
            image.put_pixel(x + 1, y,     colour_lighter);
            image.put_pixel(x,     y + 1, colour);
            image.put_pixel(x + 1, y + 1, colour);
            image.put_pixel(x,     y + 2, colour);
            image.put_pixel(x + 1, y + 2, colour);
        }

        image
    }

    fn render_45_iso(&mut self, side: &Side) -> RgbaImage {
        let size = self.size();

        match *side {
            Side::Front => self.render_45_iso_closure(
                |voxel| u32::from(voxel.z) + size.invert_x(voxel) + size.invert_y(voxel),
                |voxel| u32::from(voxel.x) + size.invert_y(voxel),
                |voxel| size.invert_z(voxel) + size.invert_x(voxel) + size.invert_y(voxel)
            ),
            Side::Left => self.render_45_iso_closure(
                |voxel| u32::from(voxel.z) + size.invert_x(voxel) + u32::from(voxel.y),
                |voxel| size.invert_x(voxel) + size.invert_y(voxel),
                |voxel| size.invert_z(voxel) + size.invert_x(voxel) + u32::from(voxel.y)
            ),
            Side::Right => self.render_45_iso_closure(
                |voxel| u32::from(voxel.z) + u32::from(voxel.x) + size.invert_y(voxel),
                |voxel| u32::from(voxel.x) + u32::from(voxel.y),
                |voxel| size.invert_z(voxel) + u32::from(voxel.x) + size.invert_y(voxel)
            ),
            Side::Back => self.render_45_iso_closure(
                |voxel| u32::from(voxel.z) + u32::from(voxel.x) + u32::from(voxel.y),
                |voxel| size.invert_x(voxel) + u32::from(voxel.y),
                |voxel| size.invert_z(voxel) + u32::from(voxel.x) + u32::from(voxel.y)
            ),
            _ => unreachable!()
        }
    }

    fn render_45_iso_closure<S, X, Y>(&mut self, sort: S, map_x: X, map_y: Y) -> RgbaImage
        where
            S: Fn(&Voxel) -> u32,
            X: Fn(&Voxel) -> u32,
            Y: Fn(&Voxel) -> u32
    {
        let mut image = self.create_image(sort, &map_x, &map_y);

        for voxel in &self.model.voxels {
            let x = map_x(voxel);
            let y = map_y(voxel);

            let colour = self.colour(voxel.i, 30);
            let colour_lighter = self.colour(voxel.i, 15);
            let colour_lightest = self.colour(voxel.i, 0);

            image.put_pixel(x,     y,     colour_lightest);
            image.put_pixel(x + 1, y,     colour_lightest);
            image.put_pixel(x,     y + 1, colour);
            image.put_pixel(x + 1, y + 1, colour_lighter);
        }

        image
    }

    fn render_22_5_iso(&mut self, side: &Side) -> RgbaImage {
        let size = self.size();

        match *side {
            Side::Front => self.render_22_5_iso_closure(
                |voxel| u32::from(voxel.z) + size.invert_x(voxel) + size.invert_y(voxel),
                |voxel| u32::from(voxel.x) * 2 + size.invert_y(voxel) * 2,
                |voxel| size.invert_z(voxel) * 3 + size.invert_x(voxel) + size.invert_y(voxel)
            ),
            Side::Left => self.render_22_5_iso_closure(
                |voxel| u32::from(voxel.z) + size.invert_x(voxel) + u32::from(voxel.y),
                |voxel| size.invert_x(voxel) * 2 + size.invert_y(voxel) * 2,
                |voxel| size.invert_z(voxel) * 3 + size.invert_x(voxel) + u32::from(voxel.y)
            ),
            Side::Right => self.render_22_5_iso_closure(
                |voxel| u32::from(voxel.z) + u32::from(voxel.x) + size.invert_y(voxel),
                |voxel| u32::from(voxel.x) * 2 + u32::from(voxel.y) * 2,
                |voxel| size.invert_z(voxel) * 3 + u32::from(voxel.x) + size.invert_y(voxel)
            ),
            Side::Back => self.render_22_5_iso_closure(
                |voxel| u32::from(voxel.z) + u32::from(voxel.x) + u32::from(voxel.y),
                |voxel| size.invert_x(voxel) * 2 + u32::from(voxel.y) * 2,
                |voxel| size.invert_z(voxel) * 3 + u32::from(voxel.x) + u32::from(voxel.y)
            ),
            _ => unreachable!()
        }
    }

    fn render_22_5_iso_closure<S, X, Y>(&mut self, sort: S, map_x: X, map_y: Y) -> RgbaImage
        where
            S: Fn(&Voxel) -> u32,
            X: Fn(&Voxel) -> u32,
            Y: Fn(&Voxel) -> u32
    {
        let mut image = self.create_image(sort, &map_x, &map_y);

        for voxel in &self.model.voxels {
            let x = map_x(voxel);
            let y = map_y(voxel);

            let colour = self.colour(voxel.i, 30);
            let colour_lighter = self.colour(voxel.i, 15);
            let colour_lightest = self.colour(voxel.i, 0);
            
            image.put_pixel(x,     y,     colour_lightest);
            image.put_pixel(x + 1, y,     colour_lightest);
            image.put_pixel(x + 2, y,     colour_lightest);
            image.put_pixel(x + 3, y,     colour_lightest);

            for y in y + 1 .. y + 4 {
                image.put_pixel(x,     y, colour);
                image.put_pixel(x + 1, y, colour);
                image.put_pixel(x + 2, y, colour_lighter);
                image.put_pixel(x + 3, y, colour_lighter);
            }
        }

        image
    }
}