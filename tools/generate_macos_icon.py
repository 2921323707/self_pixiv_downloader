#!/usr/bin/env python3
import math
import struct
import sys
import zlib
from pathlib import Path


PNG_SIGNATURE = b"\x89PNG\r\n\x1a\n"


def read_png_rgba(path):
    data = Path(path).read_bytes()
    if not data.startswith(PNG_SIGNATURE):
        raise ValueError("not a PNG file")

    pos = len(PNG_SIGNATURE)
    width = height = color_type = bit_depth = None
    idat = bytearray()

    while pos < len(data):
        length = struct.unpack(">I", data[pos : pos + 4])[0]
        kind = data[pos + 4 : pos + 8]
        chunk = data[pos + 8 : pos + 8 + length]
        pos += 12 + length

        if kind == b"IHDR":
            width, height, bit_depth, color_type, _, _, _ = struct.unpack(">IIBBBBB", chunk)
        elif kind == b"IDAT":
            idat.extend(chunk)
        elif kind == b"IEND":
            break

    if bit_depth != 8 or color_type not in (2, 6):
        raise ValueError("only 8-bit RGB/RGBA PNG files are supported")

    channels = 4 if color_type == 6 else 3
    stride = width * channels
    raw = zlib.decompress(bytes(idat))
    rows = []
    offset = 0
    prev = [0] * stride

    for _ in range(height):
        filt = raw[offset]
        offset += 1
        row = list(raw[offset : offset + stride])
        offset += stride

        for i, value in enumerate(row):
            left = row[i - channels] if i >= channels else 0
            up = prev[i]
            up_left = prev[i - channels] if i >= channels else 0
            if filt == 1:
                row[i] = (value + left) & 255
            elif filt == 2:
                row[i] = (value + up) & 255
            elif filt == 3:
                row[i] = (value + ((left + up) >> 1)) & 255
            elif filt == 4:
                predictor = paeth(left, up, up_left)
                row[i] = (value + predictor) & 255
            elif filt != 0:
                raise ValueError(f"unsupported PNG filter: {filt}")

        if channels == 3:
            rgba = []
            for i in range(0, len(row), 3):
                rgba.extend((row[i], row[i + 1], row[i + 2], 255))
            rows.append(rgba)
        else:
            rows.append(row)
        prev = row

    return width, height, rows


def paeth(a, b, c):
    p = a + b - c
    pa = abs(p - a)
    pb = abs(p - b)
    pc = abs(p - c)
    if pa <= pb and pa <= pc:
        return a
    if pb <= pc:
        return b
    return c


def write_png_rgba(path, width, height, rows):
    raw = bytearray()
    for row in rows:
        raw.append(0)
        raw.extend(row)

    def chunk(kind, payload):
        crc = zlib.crc32(kind + payload) & 0xFFFFFFFF
        return struct.pack(">I", len(payload)) + kind + payload + struct.pack(">I", crc)

    out = bytearray(PNG_SIGNATURE)
    out.extend(chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)))
    out.extend(chunk(b"IDAT", zlib.compress(bytes(raw), 9)))
    out.extend(chunk(b"IEND", b""))
    Path(path).write_bytes(out)


def sample_bilinear(src, src_w, src_h, x, y):
    x = max(0.0, min(src_w - 1.0, x))
    y = max(0.0, min(src_h - 1.0, y))
    x0 = int(math.floor(x))
    y0 = int(math.floor(y))
    x1 = min(src_w - 1, x0 + 1)
    y1 = min(src_h - 1, y0 + 1)
    tx = x - x0
    ty = y - y0
    out = []

    for c in range(4):
        p00 = src[y0][x0 * 4 + c]
        p10 = src[y0][x1 * 4 + c]
        p01 = src[y1][x0 * 4 + c]
        p11 = src[y1][x1 * 4 + c]
        top = p00 * (1 - tx) + p10 * tx
        bottom = p01 * (1 - tx) + p11 * tx
        out.append(round(top * (1 - ty) + bottom * ty))

    return out


def squircle_coverage(px, py, left, top, size, exponent=4.0):
    samples = 4
    inside = 0
    half = size / 2.0
    cx = left + half
    cy = top + half

    for sy in range(samples):
        for sx in range(samples):
            x = px + (sx + 0.5) / samples
            y = py + (sy + 0.5) / samples
            nx = abs((x - cx) / half)
            ny = abs((y - cy) / half)
            if nx**exponent + ny**exponent <= 1.0:
                inside += 1

    return inside / (samples * samples)


def compose_icon(src_path, out_path, canvas=1024, margin=112):
    src_w, src_h, src = read_png_rgba(src_path)
    size = canvas - margin * 2
    rows = [[0] * (canvas * 4) for _ in range(canvas)]

    for y in range(margin, margin + size):
        for x in range(margin, margin + size):
            cov = squircle_coverage(x, y, margin, margin, size)
            if cov <= 0:
                continue
            sx = (x - margin + 0.5) * src_w / size - 0.5
            sy = (y - margin + 0.5) * src_h / size - 0.5
            r, g, b, a = sample_bilinear(src, src_w, src_h, sx, sy)
            a = round(a * cov)
            i = x * 4
            rows[y][i : i + 4] = [r, g, b, a]

    write_png_rgba(out_path, canvas, canvas, rows)


def main():
    if len(sys.argv) != 3:
        print("usage: generate_macos_icon.py <input.png> <output.png>", file=sys.stderr)
        return 2
    compose_icon(sys.argv[1], sys.argv[2])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
