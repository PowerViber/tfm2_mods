// Generated from rules/gojo-visuals.json and unchanged body frame metadata.
pub const VERSION:usize=1;
pub const FRAMES:usize=8;
pub const FRAME_TICKS:usize=3;
pub const RENEW_TICKS:usize=6;
pub const TAIL_TICKS:usize=8;
pub const PIXEL_UNITS:i64=960;
pub fn gesture_offset(tag:&str,elapsed:usize)->[i64;2] {
let frames:&[(usize,i64,i64)]=match tag {
"gesture_blue"|"gesture_purple"=>&[(3600, -4, -7), (3600, -2, -10), (3600, 0, -10), (3600, -4, -10), (3600, -6, -8), (3600, -8, -11)],
"gesture_red"=>&[(4800, -4, -24), (4800, -4, -21), (4800, -2, -15), (4800, -4, -15), (4800, -2, -16)],
"gesture_domain"=>&[(4800, -6, -71), (4800, -8, -64), (4800, -6, -62), (4800, -6, -42), (4800, -4, -20)],
_=>return [0,0]}; let mut t=elapsed*1000%frames.iter().map(|v|v.0).sum::<usize>(); for (d,x,y) in frames {if t<*d {return [*x,*y];} t-=d;} [0,0]
}
