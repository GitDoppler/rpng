use std::{fs::File, io::Write};

use flate2::{Compression, write::ZlibEncoder};
use image::DynamicImage;

const PNG_SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

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
}

impl PngEncoder {
    fn new(width: u32, height: u32) -> Self {
        PngEncoder {
            width,
            height,
            bit_depth: 8,
            color_type: 6,
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
        let mut filtered_data = Vec::with_capacity((self.height as usize * (stride + 1)));

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
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data)?;
        encoder.finish()
    }
}

pub fn save_to_png(image: &DynamicImage, path: &str) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    let encoder = PngEncoder::new(image.width(), image.height());
    encoder.encode(image, &mut file)
}
