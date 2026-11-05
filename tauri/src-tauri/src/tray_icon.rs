//! 程序生成的托盘图标（无外部资源）：实心圆角方形色块 + 镂空 Feather 风格 git-branch，
//! 无描边（issue #1：描边版在浅色菜单栏下难辨认），右下角状态色角标（透明间隙环 + 彩点）。
//! 色块颜色跟随菜单栏深浅：深色栏白色、浅色栏深灰；镂空处透出菜单栏底色，两种外观都可读。

const CANVAS: f64 = 32.0;

// 实心色块（画布坐标）：沿用原外框几何，整体填充
const FRAME_MIN: f64 = 3.6;
const FRAME_MAX: f64 = 28.4;
const FRAME_RADIUS: f64 = 6.4;

// 镂空 git-branch（24 网格，缩小后几何中心对齐画布中心 (16,16)）
const GRID_SCALE: f64 = 0.82;
const GRID_OFFSET: f64 = 6.16; // 16 - 12 * GRID_SCALE
const GLYPH_STROKE: f64 = 2.6;

/// 角标（32 画布坐标）：透明间隙环从色块中抠出分离带，彩点压在方形块右下角上
const BADGE_X: f64 = 25.2;
const BADGE_Y: f64 = 25.2;
const BADGE_RING_R: f64 = 6.2;
const BADGE_RING_W: f64 = 1.8; // 间隙环半宽
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

/// 24 网格坐标 → 32 画布坐标（y 向下，与缓冲区行序一致）
fn g(x: f64, y: f64) -> (f64, f64) {
    (x * GRID_SCALE + GRID_OFFSET, y * GRID_SCALE + GRID_OFFSET)
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

/// 圆角方块的带符号距离（< 0 在块内）
fn frame_dist(px: f64, py: f64) -> f64 {
    let half = (FRAME_MAX - FRAME_MIN) / 2.0;
    let qx = (px - 16.0).abs() - (half - FRAME_RADIUS);
    let qy = (py - 16.0).abs() - (half - FRAME_RADIUS);
    let outside = qx.max(0.0).hypot(qy.max(0.0));
    let inside = qx.max(qy).min(0.0);
    outside + inside - FRAME_RADIUS
}

/// git-branch 图形的描边带符号距离（< 0 在描边内；作为色块的镂空区域）
fn glyph_dist(px: f64, py: f64) -> f64 {
    let half = GLYPH_STROKE / 2.0;
    // 主干竖线 (6,3)→(6,15)
    let (ax, ay) = g(6.0, 3.0);
    let (bx, by) = g(6.0, 15.0);
    let mut d = seg_dist(px, py, ax, ay, bx, by) - half;
    // 两个节点圆 (18,6) 与 (6,18)，半径 3
    for (cx, cy) in [(18.0, 6.0), (6.0, 18.0)] {
        let (ccx, ccy) = g(cx, cy);
        let dist = ((px - ccx) * (px - ccx) + (py - ccy) * (py - ccy)).sqrt();
        d = d.min((dist - 3.0 * GRID_SCALE).abs() - half);
    }
    // 连接弧：圆心 (9,9) 半径 9，从 (18,9) 扫到 (9,18)，采样为短 capsule
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

/// 渲染 size×size RGBA 托盘图标；`r/g/b` 为状态角标颜色；`dark_bar` 为菜单栏是否深色。
/// 实心色块 + 镂空：深色栏白色块、浅色栏深灰块（镂空透出菜单栏底色形成图形）。
pub fn render_tray_rgba(r: u8, gc: u8, b: u8, size: u32, dark_bar: bool) -> Vec<u8> {
    let k = size as f64 / CANVAS;
    let mut buf = vec![0u8; (size * size * 4) as usize];
    // 色块颜色：深色栏纯白；浅色栏深灰（比纯黑柔和，对比度依然充足）
    let (dr, dg, db) = if dark_bar {
        (255u8, 255u8, 255u8)
    } else {
        (58u8, 58u8, 60u8)
    };
    for y in 0..size {
        for x in 0..size {
            let px = (x as f64 + 0.5) / k;
            let py = (y as f64 + 0.5) / k;
            let (mut cr, mut cg, mut cb, mut ca) = (0u8, 0u8, 0u8, 0f64);

            // 实心圆角方块，减去镂空 git-branch 描边区域
            let disc = clamp01(0.5 - frame_dist(px, py));
            let glyph = clamp01(0.5 - glyph_dist(px, py));
            let mut base = disc * (1.0 - glyph);

            // 角标：先从色块中抠出透明间隙环，再叠状态色圆点
            let bd = ((px - BADGE_X) * (px - BADGE_X) + (py - BADGE_Y) * (py - BADGE_Y)).sqrt();
            let gap = clamp01(BADGE_RING_W - (bd - BADGE_RING_R).abs());
            base *= 1.0 - gap;
            if base > 0.0 {
                blend(&mut cr, &mut cg, &mut cb, &mut ca, dr, dg, db, base);
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

#[cfg(test)]
mod preview {
    // 渲染预览样本：cargo test dump_tray_preview && python3 scripts/rgba2png.py
    #[test]
    fn dump_tray_preview() {
        // 绿色状态样例：深色菜单栏（白色块）与浅色菜单栏（深灰块）各一张
        let samples = [
            ("/tmp/tray_preview_dark.rgba", true),
            ("/tmp/tray_preview_light.rgba", false),
        ];
        for (path, dark_bar) in samples {
            let rgba = super::render_tray_rgba(29, 185, 84, 128, dark_bar);
            // unix 保持 /tmp（配合 rgba2png.py 的固定路径）；Windows 无 /tmp，走系统临时目录
            let path = if cfg!(windows) {
                std::env::temp_dir().join(
                    std::path::Path::new(path)
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap(),
                )
            } else {
                std::path::PathBuf::from(path)
            };
            std::fs::write(&path, rgba).unwrap();
        }
    }
}
