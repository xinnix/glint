import AppKit

/// 菜单栏图标的聚合状态。
enum Aggregate: Equatable {
    case success
    case failed(Int) // 失败仓库数
    case running
    case idle
    case error
}

/// 菜单栏图标:git 分支图形 + 右下角状态角标(失败时在右侧附带失败仓库数)。
/// 与 Tauri 版托盘图标共用同一套 Feather 风格 git-branch 几何(24 网格)。
enum MenubarIcon {
    private static let height: CGFloat = 18
    private static let gridScale: CGFloat = 0.62 // 24 网格 → 画布缩放
    private static let margin: CGFloat = 1.0

    static func image(for aggregate: Aggregate) -> NSImage {
        var count: Int?
        if case .failed(let n) = aggregate, n > 0 { count = n }

        let countText: NSAttributedString?
        if let count {
            let attrs: [NSAttributedString.Key: Any] = [
                .font: NSFont.monospacedDigitSystemFont(ofSize: 11.5, weight: .semibold),
                .foregroundColor: NSColor.labelColor,
            ]
            countText = NSAttributedString(string: "\(count)", attributes: attrs)
        } else {
            countText = nil
        }
        let width = ceil(17 + (countText?.size().width ?? 0))

        let image = NSImage(size: NSSize(width: width, height: height))
        // 2x 位图 backing,保证 Retina 菜单栏渲染清晰
        let scale: CGFloat = 2
        if let rep = NSBitmapImageRep(
            bitmapDataPlanes: nil,
            pixelsWide: Int(width * scale), pixelsHigh: Int(height * scale),
            bitsPerSample: 8, samplesPerPixel: 4,
            hasAlpha: true, isPlanar: false,
            colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0
        ) {
            rep.size = NSSize(width: width, height: height)
            image.addRepresentation(rep)
        }
        image.lockFocus()
        defer { image.unlockFocus() }

        drawGlyph()
        drawBadge(for: aggregate)
        countText?.draw(at: NSPoint(x: 17, y: 3.5))

        image.isTemplate = false
        return image
    }

    /// 主图标:git 分支(主干 + 两个节点圆 + 连接弧),用标签色描边,深浅色菜单栏自适应。
    private static func drawGlyph() {
        // Feather 坐标系 y 向下,画布 y 向上,映射时翻转
        func p(_ x: CGFloat, _ y: CGFloat) -> NSPoint {
            NSPoint(x: x * gridScale + margin, y: (24 - y) * gridScale + margin)
        }

        let trunk = NSBezierPath()
        trunk.move(to: p(6, 3))
        trunk.line(to: p(6, 15))

        func nodeCircle(_ cx: CGFloat, _ cy: CGFloat) -> NSBezierPath {
            let c = p(cx, cy)
            let r = 3 * gridScale
            return NSBezierPath(ovalIn: NSRect(x: c.x - r, y: c.y - r, width: r * 2, height: r * 2))
        }

        // 连接弧:从分支圆下方 (18,9) 扫到末梢圆左侧 (9,18),圆心 (9,9)
        let arc = NSBezierPath()
        arc.appendArc(withCenter: p(9, 9), radius: 9 * gridScale,
                      startAngle: 0, endAngle: 270, clockwise: true)

        NSColor.labelColor.setStroke()
        for path in [trunk, nodeCircle(18, 6), nodeCircle(6, 18), arc] {
            path.lineWidth = 1.5
            path.lineCapStyle = .round
            path.stroke()
        }
    }

    /// 状态角标:标签色细环 + 状态色圆点,叠在图标右下角。
    private static func drawBadge(for aggregate: Aggregate) {
        let center = NSPoint(x: 14.2, y: 3.8)
        NSColor.labelColor.setFill()
        NSBezierPath(ovalIn: oval(around: center, radius: 3.5)).fill()
        badgeColor(for: aggregate).setFill()
        NSBezierPath(ovalIn: oval(around: center, radius: 2.5)).fill()
    }

    private static func badgeColor(for aggregate: Aggregate) -> NSColor {
        switch aggregate {
        case .success: return .systemGreen
        case .failed: return .systemRed
        case .running: return .systemOrange
        case .idle: return .systemGray
        case .error: return .systemPurple
        }
    }

    private static func oval(around center: NSPoint, radius: CGFloat) -> NSRect {
        NSRect(x: center.x - radius, y: center.y - radius, width: radius * 2, height: radius * 2)
    }
}
