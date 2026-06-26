use nalgebra::DMatrix;
use crate::encoder::{EncodedImage, BlockSvd};

pub fn decode_image(encoded : &EncodedImage) -> Vec<u32> {
    let w = encoded.width as usize;
    let h = encoded.height as usize;
    let block_size = encoded.block_size as usize;

    let mut raw_rgb = vec![0u32; w * h];

    let blocks_x = (w + block_size - 1) / block_size;
    let blocks_y = (h + block_size - 1) / block_size;

    let mut block_idx = 0;

    for by in 0..blocks_y {
        for bx in 0..blocks_x {
            let actual_rows = std::cmp::min(block_size, h - by * block_size);
            let actual_cols = std::cmp::min(block_size, w - bx * block_size);

            let block = &encoded.blocks[block_idx];
            block_idx += 1;

            // Blok matrislerini güvenli inşa edip çarpıyoruz
            let r_mat = decode_block_channel(&block.red, actual_rows, actual_cols);
            let g_mat = decode_block_channel(&block.green, actual_rows, actual_cols);
            let b_mat = decode_block_channel(&block.blue, actual_rows, actual_cols);

            // Çözülen bloğu ana frame buffer'a yazma
            for r in 0..actual_rows {
                let global_y = by * block_size + r;
                for c in 0..actual_cols {
                    let global_x = bx * block_size + c;

                    let red = r_mat[(r, c)].clamp(0.0, 255.0) as u32;
                    let green = g_mat[(r, c)].clamp(0.0, 255.0) as u32;
                    let blue = b_mat[(r, c)].clamp(0.0, 255.0) as u32;

                    raw_rgb[global_y * w + global_x] = (red << 16) | (green << 8) | blue;
                }
            }
        }
    }

    raw_rgb
}

fn decode_block_channel(channel : &BlockSvd, rows : usize, cols : usize) -> DMatrix<f32> {
    let k = channel.k;

    // Rebuild U matrix
    let mut u = DMatrix::zeros(rows, k);
    for c in 0..k {
        for r in 0..rows {
            // Indexing
            let idx = c * rows + r;
            if idx < channel.u_data.len() {
                u[(r, c)] = channel.u_data[idx];
            }
        }
    }

    // Revuild Vt matrix
    let mut vt = DMatrix::zeros(k, cols);
    for c in 0..cols {
        for r in 0..k {
            let idx = c * k + r;
            if idx < channel.vt_data.len() {
                vt[(r, c)] = channel.vt_data[idx];
            }
        }
    }
    
    // Rebuild Sigma diagonal matrix
    let mut sigma = DMatrix::zeros(k, k);
    for i in 0..k {
        if i < channel.sigma_data.len() {
            sigma[(i, i)] = channel.sigma_data[i];
        }
    }

    // Matris mult: A = U * Sigma * Vt
    u * sigma * vt
}
