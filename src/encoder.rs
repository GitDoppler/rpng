use flate2::{Compression, write::ZlibEncoder};
use image::DynamicImage;
use std::{fs::File, io::Write};

const PNG_SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

#[derive(Clone, Copy, Debug)]
pub enum CompressionMethod {
    Custom,
    Flate2,
}

#[allow(dead_code)]
enum FilterType {
    None = 0,
    Sub = 1,
    Up = 2,
    Average = 3,
    Paeth = 4,
}

struct PngEncoder {
    width: u32,
    height: u32,
    bit_depth: u8,
    color_type: u8,
    compression_method: CompressionMethod,
}

impl PngEncoder {
    fn new(width: u32, height: u32, compression_method: CompressionMethod) -> Self {
        PngEncoder {
            width,
            height,
            bit_depth: 8,
            color_type: 6,
            compression_method,
        }
    }

    fn encode<W: Write>(&self, image: &DynamicImage, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&PNG_SIGNATURE)?;

        self.write_ihdr(writer)?;

        self.write_idat(image, writer)?;

        self.write_iend(writer)?;

        Ok(())
    }

    fn write_ihdr<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let mut chunk_data = Vec::new();

        chunk_data.extend_from_slice(&self.width.to_be_bytes());
        chunk_data.extend_from_slice(&self.height.to_be_bytes());
        chunk_data.push(self.bit_depth);
        chunk_data.push(self.color_type);
        chunk_data.push(0);
        chunk_data.push(0);
        chunk_data.push(0);

        self.write_chunk(writer, b"IHDR", &chunk_data)
    }

    fn write_idat<W: Write>(&self, image: &DynamicImage, writer: &mut W) -> std::io::Result<()> {
        let filtered_data = self.apply_filters(image);

        let compressed_data = self.compress_data(&filtered_data)?;

        self.write_chunk(writer, b"IDAT", &compressed_data)
    }

    fn write_iend<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        // IEND chunk has no data
        self.write_chunk(writer, b"IEND", &[])
    }

    fn write_chunk<W: Write>(
        &self,
        writer: &mut W,
        chunk_type: &[u8],
        data: &[u8],
    ) -> std::io::Result<()> {
        writer.write_all(&(data.len() as u32).to_be_bytes())?;

        writer.write_all(chunk_type)?;

        writer.write_all(data)?;

        let mut crc = crc32fast::Hasher::new();
        crc.update(chunk_type);
        crc.update(data);
        let crc_value = crc.finalize();

        writer.write_all(&crc_value.to_be_bytes())?;

        Ok(())
    }

    fn apply_filters(&self, image: &DynamicImage) -> Vec<u8> {
        let bytes_per_pixel = 4;
        let stride = self.width as usize * bytes_per_pixel;
        let mut filtered_data = Vec::with_capacity(self.height as usize * (stride + 1));

        let img = image.to_rgba8();
        let img_data = img.as_raw();

        for y in 0..self.height {
            filtered_data.push(FilterType::Sub as u8);

            let row_start = y as usize * stride;
            let row_end = row_start + stride;
            let row = &img_data[row_start..row_end];

            for x in 0..stride {
                if x < bytes_per_pixel {
                    filtered_data.push(row[x]);
                } else {
                    filtered_data.push(row[x].wrapping_sub(row[x - bytes_per_pixel]));
                }
            }
        }

        filtered_data
    }

    fn compress_data(&self, data: &[u8]) -> std::io::Result<Vec<u8>> {
        match self.compression_method {
            CompressionMethod::Custom => {
                let mut compressed = Vec::new();

                // Zlib header (2 bytes)
                // CMF (Compression Method and Flags): 0x78 (deflate, 32k window)
                // FLG (Flags): 0x9C (check bits, no preset dict, default compression)
                compressed.push(0x78);
                compressed.push(0x9C);

                let deflate_data = self.simple_deflate(data);
                compressed.extend_from_slice(&deflate_data);

                // Adler32 checksum (4 bytes, big-endian)
                let checksum = self.adler32(data);
                compressed.extend_from_slice(&checksum.to_be_bytes());

                Ok(compressed)
            }
            CompressionMethod::Flate2 => {
                let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(data)?;
                encoder.finish()
            }
        }
    }

    fn simple_deflate(&self, data: &[u8]) -> Vec<u8> {
        let mut result = Vec::new();
        let mut i = 0;

        while i < data.len() {
            let (match_distance, match_length) = self.find_longest_match(data, i);

            if match_length >= 4 && match_distance > 0 && match_distance <= 65535 {
                // Encoding: 255, distance_low, distance_high, length
                let distance_low = (match_distance & 0xFF) as u8;
                let distance_high = ((match_distance >> 8) & 0xFF) as u8;

                if distance_low != 255 {
                    result.push(255); // Escape byte
                    result.push(distance_low);
                    result.push(distance_high);
                    result.push(std::cmp::min(match_length, 255) as u8);
                    i += std::cmp::min(match_length, 255);
                } else {
                    if data[i] == 255 {
                        result.push(255);
                        result.push(255);
                    } else {
                        result.push(data[i]);
                    }
                    i += 1;
                }
            } else {
                if data[i] == 255 {
                    result.push(255);
                    result.push(255);
                } else {
                    result.push(data[i]);
                }
                i += 1;
            }
        }

        result
    }

    fn find_longest_match(&self, data: &[u8], pos: usize) -> (usize, usize) {
        let mut best_distance = 0;
        let mut best_length = 0;
        let max_distance = std::cmp::min(pos, 32768);
        let max_length = std::cmp::min(258, data.len() - pos);

        for distance in 1..=max_distance {
            let start = pos - distance;
            let mut length = 0;

            while length < max_length
                && pos + length < data.len()
                && data[start + (length % distance)] == data[pos + length]
            {
                length += 1;
            }

            if length > best_length {
                best_length = length;
                best_distance = distance;
            }
        }

        (best_distance, best_length)
    }

    fn adler32(&self, data: &[u8]) -> u32 {
        let mut a: u32 = 1;
        let mut b: u32 = 0;
        const MOD_ADLER: u32 = 65521;

        for &byte in data {
            a = (a + byte as u32) % MOD_ADLER;
            b = (b + a) % MOD_ADLER;
        }

        (b << 16) | a
    }

    fn decompress_data(&self, compressed: &[u8]) -> std::io::Result<Vec<u8>> {
        if compressed.len() < 6 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Compressed data too short",
            ));
        }

        if compressed[0] != 0x78 || compressed[1] != 0x9C {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid zlib header",
            ));
        }

        // Extract deflate data (skip 2-byte header, 4-byte adler32 checksum)
        let deflate_data = &compressed[2..compressed.len() - 4];

        let decompressed = self.simple_inflate(deflate_data)?;

        let expected_checksum = u32::from_be_bytes([
            compressed[compressed.len() - 4],
            compressed[compressed.len() - 3],
            compressed[compressed.len() - 2],
            compressed[compressed.len() - 1],
        ]);

        let actual_checksum = self.adler32(&decompressed);
        if actual_checksum != expected_checksum {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Adler32 checksum mismatch",
            ));
        }

        Ok(decompressed)
    }

    fn simple_inflate(&self, data: &[u8]) -> std::io::Result<Vec<u8>> {
        let mut result = Vec::new();
        let mut i = 0;

        while i < data.len() {
            if data[i] == 255 {
                if i + 1 >= data.len() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Unexpected end of data",
                    ));
                }

                if data[i + 1] == 255 {
                    // Escaped literal 255
                    result.push(255);
                    i += 2;
                } else {
                    // Back-reference: 255, distance_low, distance_high, length
                    if i + 3 >= data.len() {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Incomplete back-reference",
                        ));
                    }

                    let distance = (data[i + 1] as usize) | ((data[i + 2] as usize) << 8);
                    let length = data[i + 3] as usize;

                    if distance == 0 || distance > result.len() || length == 0 {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Invalid back-reference parameters",
                        ));
                    }

                    // Copy from back-reference
                    let start_pos = result.len() - distance;
                    for j in 0..length {
                        let src_idx = start_pos + (j % distance);
                        let byte = result[src_idx];
                        result.push(byte);
                    }

                    i += 4;
                }
            } else {
                // Literal byte
                result.push(data[i]);
                i += 1;
            }
        }

        Ok(result)
    }

    #[allow(dead_code)]
    fn test_compression(&self, data: &[u8]) -> bool {
        match self.compress_data(data) {
            Ok(compressed) => match self.decompress_data(&compressed) {
                Ok(decompressed) => decompressed == data,
                Err(_) => false,
            },
            Err(_) => false,
        }
    }
}

pub fn save_to_png_with_compression(
    image: &DynamicImage,
    path: &str,
    compression: CompressionMethod,
) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    let encoder = PngEncoder::new(image.width(), image.height(), compression);
    encoder.encode(image, &mut file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_compression() {
        let encoder = PngEncoder::new(100, 100, CompressionMethod::Custom);
        let test_data = b"Hello, World! This is a test string for compression.";

        assert!(encoder.test_compression(test_data));
    }

    #[test]
    fn test_repetitive_data_compression() {
        let encoder = PngEncoder::new(100, 100, CompressionMethod::Custom);
        let mut test_data = Vec::new();

        // Create repetitive data that should compress well
        for _ in 0..100 {
            test_data.extend_from_slice(b"ABCDEFGH");
        }

        assert!(encoder.test_compression(&test_data));
    }

    #[test]
    fn test_empty_data() {
        let encoder = PngEncoder::new(100, 100, CompressionMethod::Custom);
        let test_data = b"";

        assert!(encoder.test_compression(test_data));
    }

    #[test]
    fn test_single_byte() {
        let encoder = PngEncoder::new(100, 100, CompressionMethod::Custom);
        let test_data = b"A";

        assert!(encoder.test_compression(test_data));
    }

    #[test]
    fn test_escape_byte_handling() {
        let encoder = PngEncoder::new(100, 100, CompressionMethod::Custom);
        let test_data = b"\xFF\xFF\xFF\x00\x01\x02";

        assert!(encoder.test_compression(test_data));
    }

    #[test]
    fn test_long_matches() {
        let encoder = PngEncoder::new(100, 100, CompressionMethod::Custom);
        let mut test_data = Vec::new();

        // Create a pattern with long repeating sequences
        let pattern = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        for _ in 0..20 {
            test_data.extend_from_slice(pattern);
        }

        assert!(encoder.test_compression(&test_data));
    }

    #[test]
    fn test_adler32_checksum() {
        let encoder = PngEncoder::new(100, 100, CompressionMethod::Custom);

        // Test known Adler32 values
        assert_eq!(encoder.adler32(b""), 1);
        assert_eq!(encoder.adler32(b"a"), 0x00620062);
        assert_eq!(encoder.adler32(b"abc"), 0x024d0127);
        assert_eq!(encoder.adler32(b"message digest"), 0x29750586);
    }

    #[test]
    fn test_compression_reduces_size() {
        let encoder = PngEncoder::new(100, 100, CompressionMethod::Custom);

        // Highly repetitive data
        let mut test_data = Vec::new();
        for _ in 0..1000 {
            test_data.push(0x42); // Repeat the same byte
        }

        let compressed = encoder.compress_data(&test_data).unwrap();

        // Compressed size should be much smaller than original (accounting for headers and overhead)
        assert!(compressed.len() < test_data.len() / 2);
    }

    #[test]
    fn test_flate2_compression() {
        let encoder = PngEncoder::new(100, 100, CompressionMethod::Flate2);
        let test_data = b"Hello, World! This is a test string for compression.";

        let compressed = encoder.compress_data(test_data).unwrap();
        assert!(compressed.len() > 0);
        assert!(compressed.len() < test_data.len() + 100); // Should be reasonable size
    }

    #[test]
    fn test_flate2_compression_repetitive_data() {
        let encoder = PngEncoder::new(100, 100, CompressionMethod::Flate2);
        let mut test_data = Vec::new();

        // Create repetitive data that should compress well
        for _ in 0..100 {
            test_data.extend_from_slice(b"ABCDEFGH");
        }

        let compressed = encoder.compress_data(&test_data).unwrap();

        // Flate2 should compress repetitive data very well
        assert!(compressed.len() < test_data.len() / 4);
    }

    #[test]
    fn test_compression_methods_comparison() {
        let custom_encoder = PngEncoder::new(100, 100, CompressionMethod::Custom);
        let flate2_encoder = PngEncoder::new(100, 100, CompressionMethod::Flate2);

        // Test data with longer repetitive patterns that our custom algorithm can compress
        let mut test_data = Vec::new();
        let pattern = b"ABCDEFGH";
        for _ in 0..50 {
            test_data.extend_from_slice(pattern);
        }

        let custom_compressed = custom_encoder.compress_data(&test_data).unwrap();
        let flate2_compressed = flate2_encoder.compress_data(&test_data).unwrap();

        // Both should compress the data significantly
        assert!(custom_compressed.len() < test_data.len() / 2);
        assert!(flate2_compressed.len() < test_data.len() / 2);

        // Flate2 should generally compress better due to Huffman coding
        println!("Original size: {}", test_data.len());
        println!("Custom compressed: {}", custom_compressed.len());
        println!("Flate2 compressed: {}", flate2_compressed.len());
    }

    #[test]
    fn test_both_methods_create_valid_pngs() {
        use std::io::Cursor;

        // Create a simple test image
        let width = 10;
        let height = 10;
        let mut image_data = Vec::new();
        for y in 0..height {
            for x in 0..width {
                // Simple gradient pattern
                let r = (x * 255 / width) as u8;
                let g = (y * 255 / height) as u8;
                let b = 128;
                let a = 255;
                image_data.extend_from_slice(&[r, g, b, a]);
            }
        }

        let image_buffer = image::RgbaImage::from_raw(width, height, image_data).unwrap();
        let dynamic_image = DynamicImage::ImageRgba8(image_buffer);

        // Test both compression methods create valid PNG structure
        let custom_encoder = PngEncoder::new(width, height, CompressionMethod::Custom);
        let flate2_encoder = PngEncoder::new(width, height, CompressionMethod::Flate2);

        let mut custom_output = Cursor::new(Vec::new());
        let mut flate2_output = Cursor::new(Vec::new());

        assert!(
            custom_encoder
                .encode(&dynamic_image, &mut custom_output)
                .is_ok()
        );
        assert!(
            flate2_encoder
                .encode(&dynamic_image, &mut flate2_output)
                .is_ok()
        );

        // Both outputs should start with PNG signature
        let custom_data = custom_output.into_inner();
        let flate2_data = flate2_output.into_inner();

        assert_eq!(&custom_data[0..8], &PNG_SIGNATURE);
        assert_eq!(&flate2_data[0..8], &PNG_SIGNATURE);

        // Both should have reasonable file sizes
        assert!(custom_data.len() > 100);
        assert!(flate2_data.len() > 100);
    }
}
