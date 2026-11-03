// 渲染产品应用图标（1024×1024 PNG）：深色圆角方形 + Feather 风格 git-branch 白描边 + 状态绿点角标。
// 几何与托盘图标同源（24 网格：主干 (6,3)-(6,15)、节点圆 (18,6)/(6,18) r3、连接弧圆心 (9,9) r9）。
// 用法：swift scripts/render-appicon.swift <输出路径.png>
import CoreGraphics
import ImageIO
import Foundation
import UniformTypeIdentifiers

let SIZE = 1024
let CORNER = CGFloat(230) // macOS 圆角比例 ~22.5%

// 24 网格 → 图标坐标（网格 (3,3)-(21,21) 的包围盒居中，占画布 ~52%）
let scale = CGFloat(SIZE) * 0.52 / 18.0
let offset = (CGFloat(SIZE) - 18.0 * scale) / 2.0 - 3.0 * scale
func g(_ v: CGFloat) -> CGFloat { v * scale + offset }
let stroke = 2.0 * scale // Feather 24 网格标准 2 单位描边

let colorSpace = CGColorSpace(name: CGColorSpace.sRGB)!
let ctx = CGContext(data: nil, width: SIZE, height: SIZE, bitsPerComponent: 8, bytesPerRow: 0,
                    space: colorSpace, bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue)!

// y 向下坐标系（与 Feather 网格一致）
ctx.translateBy(x: 0, y: CGFloat(SIZE))
ctx.scaleBy(x: 1, y: -1)

// 背景：深色圆角方形 + 自上而下的微渐变
let bg = CGPath(roundedRect: CGRect(x: 0, y: 0, width: SIZE, height: SIZE),
                cornerWidth: CORNER, cornerHeight: CORNER, transform: nil)
ctx.addPath(bg)
ctx.clip()
let colors = [
    CGColor(red: 0.205, green: 0.205, blue: 0.225, alpha: 1),
    CGColor(red: 0.125, green: 0.125, blue: 0.14, alpha: 1),
] as CFArray
if let grad = CGGradient(colorsSpace: colorSpace, colors: colors, locations: [0, 1]) {
    ctx.drawLinearGradient(grad, start: .zero, end: CGPoint(x: 0, y: CGFloat(SIZE)), options: [])
}

// git-branch 白描边（圆头）
ctx.setStrokeColor(CGColor(red: 1, green: 1, blue: 1, alpha: 1))
ctx.setLineWidth(stroke)
ctx.setLineCap(.round)
ctx.setLineJoin(.round)

// 主干 (6,3)→(6,15)
ctx.move(to: CGPoint(x: g(6), y: g(3)))
ctx.addLine(to: CGPoint(x: g(6), y: g(15)))
ctx.strokePath()

// 节点圆 (18,6) r3、(6,18) r3
for (cx, cy) in [(CGFloat(18), CGFloat(6)), (CGFloat(6), CGFloat(18))] {
    ctx.addEllipse(in: CGRect(x: g(cx) - 3 * scale, y: g(cy) - 3 * scale,
                              width: 6 * scale, height: 6 * scale))
    ctx.strokePath()
}

// 连接弧：圆心 (9,9) r9，从 (18,9) 扫到 (9,18)——按托盘同款方式采样为折线，规避翻转坐标系的弧向歧义
let arcC = CGPoint(x: g(9), y: g(9))
let r = 9.0 * scale
let steps = 24
var prev = CGPoint(x: arcC.x + r, y: arcC.y)
ctx.move(to: prev)
for i in 1...steps {
    let th = CGFloat.pi / 2 * CGFloat(i) / CGFloat(steps)
    let p = CGPoint(x: arcC.x + r * cos(th), y: arcC.y + r * sin(th))
    ctx.addLine(to: p)
    prev = p
}
ctx.strokePath()

// 状态角标：白环 + 绿点（产品主色 #1db954），位于右下
let badge = CGPoint(x: 756, y: 756)
ctx.setFillColor(CGColor(red: 1, green: 1, blue: 1, alpha: 1))
ctx.addEllipse(in: CGRect(x: badge.x - 134, y: badge.y - 134, width: 268, height: 268))
ctx.fillPath()
ctx.setFillColor(CGColor(red: 0.114, green: 0.725, blue: 0.329, alpha: 1))
ctx.addEllipse(in: CGRect(x: badge.x - 112, y: badge.y - 112, width: 224, height: 224))
ctx.fillPath()

// 输出 PNG
guard CommandLine.arguments.count > 1 else {
    FileHandle.standardError.write("用法：swift render-appicon.swift <输出.png>\n".data(using: .utf8)!)
    exit(2)
}
let out = CommandLine.arguments[1]
let image = ctx.makeImage()!
let dest = CGImageDestinationCreateWithURL(URL(fileURLWithPath: out) as CFURL, UTType.png.identifier as CFString, 1, nil)!
CGImageDestinationAddImage(dest, image, nil)
guard CGImageDestinationFinalize(dest) else {
    FileHandle.standardError.write("写入 PNG 失败: \(out)\n".data(using: .utf8)!)
    exit(1)
}
print("已生成 \(out)")
