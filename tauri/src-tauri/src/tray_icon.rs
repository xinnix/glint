//! 程序生成的托盘图标(无外部资源):Feather 风格 git-branch 图形 + 右下角状态角标。
//! 几何与原生 Swift 版 MenubarIcon 保持一致(24 网格),主图形为白色描边并带
//! 半透明深色外沿,深浅色菜单栏/任务栏下均可见。

const CANVAS: f64 = 32.0;
const GRID_SCALE: f64 = 1.10; // 24 网格 → 32 画布
const MARGIN: f64 = 1.8;
const STROKE: f64 = 2.4;

/// 角标(32 画布坐标):与 Swift 版 (14.2, 3.8)/18pt 对应
const BADGE_X: f64 = 25.2;
const BADGE_Y: f64 = 25.2;
const BADGE_RING_R: f64 = 6.2;
const BADGE_DOT_R: f64 = 4.4;

fn clamp01(v: f64) -> f64 {
    if v < 0.0 {
        0.0
    } else if v > 1.0 {
        1.0
    } else {
        v
    }
}

/// 24 网格坐标 → 32 画布坐标(y 向下,与缓冲区行序一致)
fn g(x: f64, y: f64) -> (f64, f64) {
    (x * GRID_SCALE + MARGIN, y * GRID_SCALE + MARGIN)
}

fn seg_dist(px: f64, py: f64, ax: f64, ay: f64, bx: f64, by: f64) -> f64 {
    let vx = bx - ax;
    let vy = by - ay;
    let len2 = vx * vx + vy * vy;
    let t = if len2 == 0.0 {
        0.0
    } else {
        (((px - ax) * vx + (py - ay) * vy) / len2).clamp(0.0, 1.0)
    };
    let dx = px - (ax + t * vx);
    let dy = py - (ay + t * vy);
    (dx * dx + dy * dy).sqrt()
}

/// git-branch 图形的带符号距离(< 0 在描边内部)
fn glyph_dist(px: f64, py: f64) -> f64 {
    let half = STROKE / 2.0;
    // 主干竖线 (6,3)→(6,15)
    let (ax, ay) = g(6.0, 3.0);
    let (bx, by) = g(6.0, 15.0);
    let mut d = seg_dist(px, py, ax, ay, bx, by) - half;
    // 两个节点圆 (18,6) 与 (6,18),半径 3
    for (cx, cy) in [(18.0, 6.0), (6.0, 18.0)] {
        let (ccx, ccy) = g(cx, cy);
        let dist = ((px - ccx) * (px - ccx) + (py - ccy) * (py - ccy)).sqrt();
        d = d.min((dist - 3.0 * GRID_SCALE).abs() - half);
    }
    // 连接弧:圆心 (9,9) 半径 9,从 (18,9) 扫到 (9,18),采样为短 capsule
    let (ccx, ccy) = g(9.0, 9.0);
    let r = 9.0 * GRID_SCALE;
    let steps = 12;
    let (mut px0, mut py0) = (ccx + r, ccy);
    for i in 1..=steps {
        let th = std::f64::consts::FRAC_PI_2 * i as f64 / steps as f64;
        let (px1, py1) = (ccx + r * th.cos(), ccy + r * th.sin());
        d = d.min(seg_dist(px, py, px0, py0, px1, py1) - half);
        px0 = px1;
        py0 = py1;
    }
    d
}

/// src-over 颜色混合
fn blend(cr: &mut u8, cg: &mut u8, cb: &mut u8, ca: &mut f64, r: u8, g: u8, b: u8, a: f64) {
    let oa = a + *ca * (1.0 - a);
    if oa <= 0.0 {
        return;
    }
    *cr = ((r as f64 * a + *cr as f64 * *ca * (1.0 - a)) / oa).round() as u8;
    *cg = ((g as f64 * a + *cg as f64 * *ca * (1.0 - a)) / oa).round() as u8;
    *cb = ((b as f64 * a + *cb as f64 * *ca * (1.0 - a)) / oa).round() as u8;
    *ca = oa;
}

/// 渲染 size×size RGBA 托盘图标;`r/g/b` 为状态角标颜色。
/// `dark`=深色菜单栏(白色图形+深色外沿);浅色菜单栏则反转(深色图形+白色外沿)。
pub fn render_tray_rgba(r: u8, gc: u8, b: u8, size: u32, dark: bool) -> Vec<u8> {
    let k = size as f64 / CANVAS;
    let mut buf = vec![0u8; (size * size * 4) as usize];
    // 深色栏:白图形+黑外沿;浅色栏:近黑图形+白外沿
    let (gr, gg, gb) = if dark { (255u8, 255u8, 255u8) } else { (29u8, 29u8, 31u8) };
    let (rr, rg, rb) = if dark { (0u8, 0u8, 0u8) } else { (255u8, 255u8, 255u8) };
    for y in 0..size {
        for x in 0..size {
            let px = (x as f64 + 0.5) / k;
            let py = (y as f64 + 0.5) / k;
            let (mut cr, mut cg, mut cb, mut ca) = (0u8, 0u8, 0u8, 0f64);

            // 半透明外沿:保证图形在相反明度的菜单栏下可见
            let d = glyph_dist(px, py);
            let rim = clamp01(1.4 - d) * clamp01(d + 0.5) * 0.55;
            if rim > 0.0 {
                blend(&mut cr, &mut cg, &mut cb, &mut ca, rr, rg, rb, rim);
            }
            // 主图形
            let fill = clamp01(0.5 - d);
            if fill > 0.0 {
                blend(&mut cr, &mut cg, &mut cb, &mut ca, gr, gg, gb, fill);
            }
            // 角标:白环 + 状态色圆点
            let bd = ((px - BADGE_X) * (px - BADGE_X) + (py - BADGE_Y) * (py - BADGE_Y)).sqrt();
            let ring = clamp01(0.5 - (bd - BADGE_RING_R));
            if ring > 0.0 {
                blend(&mut cr, &mut cg, &mut cb, &mut ca, 255, 255, 255, ring);
            }
            let dot = clamp01(0.5 - (bd - BADGE_DOT_R));
            if dot > 0.0 {
                blend(&mut cr, &mut cg, &mut cb, &mut ca, r, gc, b, dot);
            }

            let i = ((y * size + x) * 4) as usize;
            buf[i] = cr;
            buf[i + 1] = cg;
            buf[i + 2] = cb;
            buf[i + 3] = (ca * 255.0).round() as u8;
        }
    }
    buf
}
