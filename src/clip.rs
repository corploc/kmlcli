const INSIDE: u8 = 0;
const LEFT: u8 = 1;
const RIGHT: u8 = 2;
const BOTTOM: u8 = 4;
const TOP: u8 = 8;

fn outcode(x: f64, y: f64, x_bounds: &[f64; 2], y_bounds: &[f64; 2]) -> u8 {
    let mut code = INSIDE;
    if x < x_bounds[0] {
        code |= LEFT;
    } else if x > x_bounds[1] {
        code |= RIGHT;
    }
    if y < y_bounds[0] {
        code |= BOTTOM;
    } else if y > y_bounds[1] {
        code |= TOP;
    }
    code
}

pub fn clip_line(
    mut x1: f64,
    mut y1: f64,
    mut x2: f64,
    mut y2: f64,
    x_bounds: &[f64; 2],
    y_bounds: &[f64; 2],
) -> Option<(f64, f64, f64, f64)> {
    let mut code1 = outcode(x1, y1, x_bounds, y_bounds);
    let mut code2 = outcode(x2, y2, x_bounds, y_bounds);

    loop {
        if (code1 | code2) == 0 {
            return Some((x1, y1, x2, y2));
        }
        if (code1 & code2) != 0 {
            return None;
        }

        let code_out = if code1 != 0 { code1 } else { code2 };
        let dx = x2 - x1;
        let dy = y2 - y1;

        let (x, y);
        if code_out & TOP != 0 {
            x = x1 + dx * (y_bounds[1] - y1) / dy;
            y = y_bounds[1];
        } else if code_out & BOTTOM != 0 {
            x = x1 + dx * (y_bounds[0] - y1) / dy;
            y = y_bounds[0];
        } else if code_out & RIGHT != 0 {
            y = y1 + dy * (x_bounds[1] - x1) / dx;
            x = x_bounds[1];
        } else {
            y = y1 + dy * (x_bounds[0] - x1) / dx;
            x = x_bounds[0];
        }

        if code_out == code1 {
            x1 = x;
            y1 = y;
            code1 = outcode(x1, y1, x_bounds, y_bounds);
        } else {
            x2 = x;
            y2 = y;
            code2 = outcode(x2, y2, x_bounds, y_bounds);
        }
    }
}
