#!/usr/bin/env python3
"""把 render_tray_rgba 输出的原始 RGBA 缓冲转成 PNG 预览。

用法：cargo test dump_tray_preview && python3 scripts/rgba2png.py [FILE...]
无参数时转换两个默认样本（深色/浅色菜单栏）。仅依赖标准库。
"""
import struct
import sys
import zlib
from pathlib import Path

DEFAULTS = ["/tmp/tray_preview_dark.rgba", "/tmp/tray_preview_light.rgba"]


def chunk(tag: bytes, data: bytes) -> bytes:
    return (
        struct.pack(">I", len(data))
        + tag
        + data
        + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)
    )


def write_png(path: Path, side: int, rgba: bytes) -> None:
    # 每行前置 filter 字节 0（None）
    raw = b"".join(b"\x00" + rgba[y * side * 4 : (y + 1) * side * 4] for y in range(side))
    png = (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", struct.pack(">IIBBBBB", side, side, 8, 6, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress(raw, 9))
        + chunk(b"IEND", b"")
    )
    out = path.with_suffix(".png")
    out.write_bytes(png)
    print(f"{path} -> {out} ({side}x{side})")


def main(argv: list[str]) -> None:
    files = [Path(a) for a in argv] or [Path(p) for p in DEFAULTS]
    for path in files:
        data = path.read_bytes()
        n = len(data) // 4
        side = round(n**0.5)
        if side * side != n:
            raise SystemExit(f"{path}: {len(data)} 字节不是正方形 RGBA 缓冲")
        write_png(path, side, data)


if __name__ == "__main__":
    main(sys.argv[1:])
