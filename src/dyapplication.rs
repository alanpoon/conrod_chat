use libloading::Library;
use conrod::{position, Positionable};
use conrod::Rect;
use conrod::widget::Rectangle;
use std;
pub struct Application(pub Library);

#[derive(Clone,Copy,PartialEq,Debug)]
pub struct SpriteInfo {
    pub first: (f64, f64), //left corner of first
    pub num_in_row: f64,
    pub w_h: (f64, f64),
    pub pad: (f64, f64, f64, f64),
}
impl SpriteInfo {
    pub fn src_rect(&self, index: f64) -> Rect {
        let s = self;
        let (x, y) = (index % s.num_in_row as f64, (index / (s.num_in_row)).floor());
        let r = position::rect::Rect::from_corners([s.first.0 + x * s.w_h.0 + s.pad.0,
                                                    s.first.1 - y * s.w_h.1 - s.pad.2],
                                                   [s.first.0 + (x + 1.0) * s.w_h.0 - s.pad.1,
                                                    s.first.1 - (y + 1.0) * s.w_h.1 + s.pad.3]);
        r
    }
}
#[repr(C)]
#[derive(Clone,Copy,PartialEq,Debug)]
pub struct RGB(pub f32, pub f32, pub f32, pub f32);

#[repr(C)]
#[derive(Clone,PartialEq,Debug)]
pub struct Static_Style {
    pub w_h: (f64, f64),
    pub rect: (RGB, f64, f64, f64), //w,h, pad bottom
    pub image: (SpriteInfo, f64, f64, f64, f64), // w,h,l,t
    pub text: (u32, RGB, f64, f64, f64, f64), // fontsize,RGB,w,h,l,t
}
impl Application {
    pub fn new(libpath:&'static str)->Application{
        Application(Library::new(libpath).unwrap_or_else(|error| panic!("{}", error)))
    }
    pub fn in_loop(&mut self,libpath:&'static str,last_modified:&mut std::time::Instant)->Self{
                if let Ok(Ok(modified)) = std::fs::metadata(libpath).map(|m| m.modified()) {
            if modified > last_modified {
                 drop(self);
                  last_modified = modified;
        Application(Library::new(libpath).unwrap_or_else(|error| panic!("{}", error)))
            }
            self
       
    }
    pub fn get_static_styles(&self) -> Static_Style {
        unsafe {
            let f = self.0.get::<fn() -> Static_Style>(b"get_static_styles\0").unwrap();
            f()
        }
    }
}
