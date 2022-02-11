use quicli::prelude::*;
use structopt::StructOpt;
// use std::path::{Path, PathBuf};

///
/// Program img-bri-groups searches for bright regions in BW-png and output their coordinates
///
#[derive(StructOpt, Debug)]
#[structopt(name = "img-bri-groups")]
struct Cli {
    /// The target PNG-file, should be b/w.
    target: String,

    #[structopt(flatten)]
    verbosity: Verbosity,

//    /// How many pixels from border to skip?
//    #[structopt(short, long, default_value = "0")]
//    border: u32,

    /// From input to output level threshold to cut off
    #[structopt(short, long, default_value = "40")]
    threshold: u8,

    /// Region size: usually size of center of sample
    #[structopt(short = "s", default_value = "5")]
    region_size: u32,

    /// Force to continue
    #[structopt(short, long)]
    forced: bool,
}

/// Finds regions
fn main() -> CliResult {
    let args = Cli::from_args();
    args.verbosity.setup_env_logger(&env!("CARGO_PKG_NAME"))?;

    // dbg!(args);
    info!("Reading {:?}...", args.target);
    let img = read_image(&args.target)?;

    info!("Starts search on bright regions in {:?}...", args.target);

    let img = img.to_luma8();
    let mut rgns = Vec::<Region>::new();
    let mut pxls = Vec::<Point>::new();

    // first get non-0 points
    for (x, y, v) in img.enumerate_pixels() {
        if v[0] > args.threshold {
            // dbg!(x,y,v);
            pxls.push(Point {
                x, y, v:v[0],
            });
        }
    }

    // assert_eq!(ppp.is_within(&rrr), false);

    match ! pxls.is_empty() {
        true => {
            if pxls.len() > 1000 && !args.forced {
                eprintln!("Got too much bright pixels ({} > 1000).", pxls.len());
                assert!(args.forced, "Use -f to force calculation.");
            }
            // 2. {r}, {p}

            info!("{}:{}", ".".repeat(pxls.len()), pxls.len());

            while ! pxls.is_empty() {
                for r in rgns.iter_mut() {
                    // 3. r <- {p}
                    let rrr = r.clone();
                    for p in pxls.iter().filter(|p| p.is_within(&rrr)) {
                        r.expand(*p);
                        // dbg!("EXPANDED:",p.clone(),rrr.clone(),r.clone());
                    }

                    // 4. {r} -> {p}
                    pxls.retain(|&p| !p.is_within(r));
                }

                if let Some(p) = pxls.pop() {
                    rgns.push(Region::new(&p, &p, args.region_size));
                }
            }

            for r in rgns.iter() {
                println!("[ {} : {} : {} ]", r.c.x, r.c.y, r.c.v);
            }

            eprintln!("{:?} finished successfully", args.target);
            Ok(())
        }
        false => {
            eprintln!(
                "No pixels in {:?} brighter than {}",
                args.target, args.threshold
            );
            Ok(())
        }
    }
}

fn read_image(thefilename: &String) -> image::ImageResult<image::DynamicImage> {
    let img = image::open(std::path::PathBuf::from(thefilename).as_path())?;
    Ok(img)
}

#[derive(Clone, Debug, Copy)]
pub struct Point {
    x: u32,
    y: u32,
    v: u8,
}
impl Point {
    fn is_within(&self, r: &Region) -> bool {
        let c = r.c;
        let (l, r, t, b) = (
            c.x - r.rs / 2,
            c.x + r.rs / 2,
            c.y - r.rs / 2,
            c.y + r.rs / 2,
        );
        if (l <= self.x) && (self.x <= r) && (t <= self.y) && (self.y <= b) {
            true // point is within region bounds + region_size
        } else {
            false // point is farther than region_size
        }
    }
}

#[derive(Clone, Debug)]
pub struct Region {
    tl: Point,
    br: Point,
    c: Point,
    rs: u32,
}
impl Region {
    fn new(tl: &Point, br: &Point, rs: u32) -> Region {
        let mut r: Region = Region {
            tl: Point {
                x: tl.x,
                y: tl.y,
                v: tl.v,
            },
            br: Point {
                x: br.x,
                y: br.y,
                v: br.v,
            },
            c: Point { x: 0, y: 0, v: 0 },
            rs,
        };
        r.center();
        r
    }
    fn center(&mut self) -> Point {
        self.c.x = (self.tl.x + self.br.x) / 2;
        self.c.y = (self.tl.y + self.br.y) / 2;
        self.c.v = (self.tl.v + self.br.v) / 2;
        self.c
    }
    fn glue(&mut self, p: Point) -> Point {
        if p.x < self.tl.x {
            self.tl.x = p.x;
        }
        if p.x > self.br.x {
            self.br.x = p.x;
        }
        if p.y < self.tl.y {
            self.tl.y = p.y;
        }
        if p.y > self.br.y {
            self.br.y = p.y;
        }
        self.c.v = (self.c.v + p.v) / 2;
        self.center()
    }
    fn expand(&mut self, p: Point) -> Point {
        // alias
        self.glue(p)
    }
}

