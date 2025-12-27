import struct

# Your exact matrix (top-down)
pixels = [[4, 6, 3], [5, 3, 12], [9, 3, 5]]

width = 3
height = 3
row_size = width + 1  # 3 bytes data + 1 padding byte
data_size = row_size * height
palette_size = 1024  # 256 colors * 4 bytes each
offset = 14 + 40 + palette_size
file_size = offset + data_size

with open("test_3x3_custom.bmp", "wb") as f:
    # File header (14 bytes)
    f.write(b"BM")
    f.write(struct.pack("<I", file_size))
    f.write(struct.pack("<I", 0))
    f.write(struct.pack("<I", offset))

    # DIB header (BITMAPINFOHEADER, 40 bytes)
    f.write(struct.pack("<I", 40))
    f.write(struct.pack("<i", width))
    f.write(struct.pack("<i", height))
    f.write(struct.pack("<H", 1))
    f.write(struct.pack("<H", 8))
    f.write(struct.pack("<I", 0))
    f.write(struct.pack("<I", data_size))
    f.write(struct.pack("<i", 2835))
    f.write(struct.pack("<i", 2835))
    f.write(struct.pack("<I", 256))
    f.write(struct.pack("<I", 256))

    # Grayscale palette
    for i in range(256):
        f.write(bytes([i, i, i, 0]))

    # Pixel data (BMP stores rows bottom-up, so reverse)
    for row in reversed(pixels):
        for val in row:
            f.write(bytes([val]))
        f.write(b"\x00")  # padding byte

print("Created test_3x3_custom.bmp")
