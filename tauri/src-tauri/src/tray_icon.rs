//! 程序生成的托盘图标（无外部资源）：镂空圆角方形外框 + 内嵌 Feather 风格 git-branch，
//! 全部为描边线条（无填充），右下角状态色角标（白环 + 彩点）。
//! 图形恒定白色 + 半透明深色外沿：深色菜单栏下直接可见，浅色菜单栏靠外沿保证对比。

const CANVAS: f64 = 32.0;

// 方形外框（画布坐标）
const FRAME_MIN: f64 = 3.6;
const FRAME_MAX: f64 = 28.4;
const FRAME_RADIUS: f64 = 6.4;
const FRAME_STROKE: f64 = 1.7;

// 内嵌 git-branch（24 网格，缩小后几何中心对齐画布中心 (16,16)）
const GRID_SCALE: f64 = 0.82;
const GRID_OFFSET: f64 = 6.16; // 16 - 12 * GRID_SCALE
const GLYPH_STROKE: f64 = 2.2;

/// 角标（32 画布坐标）：白环 + 状态色圆点，压在方形框右下角上
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

/// 镂空圆角方形外框的描边带符号距离（< 0 在描边内）
fn frame_dist(px: f64, py: f64) -> f64 {
    let half = (FRAME_MAX - FRAME_MIN) / 2.0;
    let qx = (px - 16.0).abs() - (half - FRAME_RADIUS);
    let qy = (py - 16.0).abs() - (half - FRAME_RADIUS);
    let outside = qx.max(0.0).hypot(qy.max(0.0));
    let inside = qx.max(qy).min(0.0);
    let d = outside + inside - FRAME_RADIUS; // 圆角矩形边缘的有符号距离
    d.abs() - FRAME_STROKE / 2.0
}

/// git-branch 图形的描边带符号距离（< 0 在描边内）
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

/// 渲染 size×size RGBA 托盘图标；`r/g/b` 为状态角标颜色。
/// 图形恒定白色，带半透明深色外沿（浅色菜单栏下的对比度兜底）。
pub fn render_tray_rgba(r: u8, gc: u8, b: u8, size: u32) -> Vec<u8> {
    let k = size as f64 / CANVAS;
    let mut buf = vec![0u8; (size * size * 4) as usize];
    for y in 0..size {
        for x in 0..size {
            let px = (x as f64 + 0.5) / k;
            let py = (y as f64 + 0.5) / k;
            let (mut cr, mut cg, mut cb, mut ca) = (0u8, 0u8, 0u8, 0f64);

            // 外框 ∪ git-branch 的描边距离
            let d = frame_dist(px, py).min(glyph_dist(px, py));
            // 半透明深色外沿：浅色菜单栏下保证白色线条可见
            let rim = clamp01(1.4 - d) * clamp01(d + 0.5) * 0.55;
            if rim > 0.0 {
                blend(&mut cr, &mut cg, &mut cb, &mut ca, 0, 0, 0, rim);
            }
            // 白色镂空线条
            let fill = clamp01(0.5 - d);
            if fill > 0.0 {
                blend(&mut cr, &mut cg, &mut cb, &mut ca, 255, 255, 255, fill);
            }
            // 角标：白环 + 状态色圆点
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

#[cfg(test)]
mod preview {
    // 渲染预览样本：cargo test dump_tray_preview && python3 scripts/rgba2png.py
    #[test]
    fn dump_tray_preview() {
        let rgba = super::render_tray_rgba(29, 185, 84, 128); // 绿色状态样例
        // unix 保持 /tmp（配合 rgba2png.py 的固定路径）；Windows 无 /tmp，走系统临时目录
        let path = if cfg!(windows) {
            std::env::temp_dir().join("tray_preview.rgba")
        } else {
            std::path::PathBuf::from("/tmp/tray_preview.rgba")
        };
        std::fs::write(&path, rgba).unwrap();
    }
}
