#[no_mangle]
pub  extern "C" fn get_static_styles()->Static_Style{
    Static_Style{
      w_h:(300.0,40.0),
      rect:(RGB(0.40,0.15,0.20,1.0),300.0,40.0,2.0),
    image:(SpriteInfo{
      first:(0.0,270.0),
      num_in_row:4.0,
      w_h:(150.0,90.0),
      pad:(10.0,10.0,0.0,0.0)
    },20.0,20.0,5.0,5.0),
    text:(18,RGB(0.82,0.27,0.25,1.0),100.0,50.0,22.0,5.0),
  }
}
#[repr(C)]
pub struct SpriteInfo{
  first:(f64,f64), //left corner of first
  num_in_row:f64,
  w_h:(f64,f64),
  pad:(f64,f64,f64,f64),
}
#[repr(C)]
pub struct RGB(f32,f32,f32,f32);

pub struct Static_Style{
    pub w_h: (f64, f64),
    pub rect: (RGB,f64, f64,f64), //w,h, pad bottom
    pub image: (SpriteInfo, f64, f64, f64, f64), // w,h,l,t
    pub text:(u32, RGB, f64, f64, f64, f64), // fontsize,RGB,w,h,l,t
}