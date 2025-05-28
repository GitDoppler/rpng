# rpng

A Rust-based CLI tool for converting images to PNG format with manual compression implementation. This project demonstrates fundamental lossless data compression concepts by implementing a simplified LZ77-based algorithm wrapped in zlib format, alongside standard compression for comparison.

## Features

- **Multi-format Support**: Convert various image formats (JPEG, WebP, BMP, etc.) to PNG
- **Dual Compression Methods**: 
  - Custom simplified DEFLATE implementation (educational)
  - Standard flate2 DEFLATE implementation (production-ready)
- **PNG Compliance**: Generates fully compliant PNG files with proper structure and checksums
- **Sub Filtering**: Applies PNG Sub filtering to improve compression efficiency
- **Educational Value**: Transparent implementation showcasing compression fundamentals

## Installation

### Prerequisites
- Rust 1.70 or later
- Cargo package manager

### Build from Source
```bash
git clone https://github.com/yourusername/rpng.git
cd rpng
cargo build --release
```

The compiled binary will be available at `target/release/rpng`.

## Usage

### Basic Usage
```bash
# Convert using custom compression (default)
rpng input.jpg

# Specify output path
rpng input.jpg output.png

# Use standard flate2 compression
rpng --flate2 input.jpg output.png

# Use custom compression explicitly
rpng --custom input.jpg output.png
```

### Command Line Options
```
rpng [--custom|--flate2] <image_path> [output_path]

Compression Methods:
  --custom  Use our custom simplified DEFLATE algorithm (default)
  --flate2  Use the standard flate2 DEFLATE implementation

Examples:
  rpng photo.jpg                        # Use custom compression
  rpng --custom photo.jpg output.png    # Custom compression with output path
  rpng --flate2 photo.jpg output.png    # Standard compression
```

## Technical Implementation

### Custom LZ77 Algorithm
Our simplified compression implementation features:

- **Sliding Window**: Up to 32KB lookback distance
- **Match Detection**: Finds repeated sequences of 3-255 bytes
- **Encoding Scheme**: 
  - Bytes 0-254: Stored as literals
  - Byte 255: Escape sequence for encoding back-references
  - Format: `255, distance_low, distance_high, length`

### Zlib Container
The compressed data uses standard zlib format:
- **Header**: `0x78 0x9C` (deflate compression, 32K window)
- **Payload**: Custom LZ77-compressed data
- **Checksum**: Adler32 checksum for integrity verification

### PNG Structure
Generated files include:
- PNG signature (8 bytes)
- IHDR chunk (image metadata)
- IDAT chunk (compressed image data)
- IEND chunk (end marker)
- CRC32 checksums for each chunk

## Performance Comparison

Testing with a 145KB Saturn image:

| Method | Output Size | Compression Ratio | Space Savings |
|--------|-------------|-------------------|---------------|
| Original | 145 KB | - | - |
| Custom Algorithm | 91 KB | 62.8% | 37.2% |
| Flate2 Algorithm | 52 KB | 35.9% | 64.1% |

The flate2 implementation achieves ~75% better compression due to Huffman coding, while our custom algorithm successfully demonstrates core LZ77 concepts with meaningful compression.

## Development

### Running Tests
```bash
cargo test
```

### Test Coverage
The project includes comprehensive tests for:
- Basic compression/decompression
- Repetitive data patterns
- Edge cases (empty data, single bytes)
- Escape sequence handling
- Long match detection
- Adler32 checksum verification
- Compression effectiveness
- PNG format compliance

### Dependencies
- `image`: Image decoding and format support
- `flate2`: Standard DEFLATE implementation for comparison
- `crc32fast`: CRC32 checksum calculation

## Educational Aspects

This project serves as a practical demonstration of:
- **LZ77 Compression**: Dictionary-based compression using sliding windows
- **PNG Format**: Understanding PNG file structure and chunk organization
- **Zlib Format**: Container format with headers and checksums
- **Data Filtering**: PNG Sub filtering for improved compression
- **Checksum Algorithms**: Adler32 and CRC32 for data integrity

## Limitations

- **Performance**: O(n²) worst-case time complexity due to linear window search
- **Compression Ratio**: Custom algorithm lacks Huffman coding, resulting in larger files than standard DEFLATE
- **Optimization**: Simplified implementation prioritizes clarity over maximum efficiency

## Contributing

Contributions are welcome! Areas for improvement:
- Huffman coding implementation for better compression
- Optimized match-finding algorithms (hash tables, suffix arrays)
- Additional PNG filter types (Up, Average, Paeth)
- Performance benchmarking and optimization

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- PNG specification and format documentation
- LZ77 algorithm research and implementation references
- Rust image processing community and crate ecosystem

---

**Note**: This implementation is primarily educational and demonstrates compression concepts. For production use, consider using established libraries like `image` with standard compression methods.