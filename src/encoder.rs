use nalgebra::DMatrix;
use serde::{Serialize, Deserialize};

const BLOCK_SIZE : usize = 16;

#[derive(Serialize, Deserialize)]
pub struct BlockSvd {
    pub k : usize,
    pub u_data : Vec<i8>,
    pub sigma_data : Vec<u8>,
    pub sigma_max : f32,
    pub vt_data : Vec<i8>,
}

#[derive(Serialize, Deserialize)]
pub struct EncodedBlock {
    pub red : BlockSvd,
    pub green : BlockSvd,
    pub blue : BlockSvd,
}

#[derive(Serialize, Deserialize)]
pub struct EncodedImage {
    pub width : u32,
    pub height : u32,
    pub block_size : u32,
    pub blocks : Vec<EncodedBlock>,
}

// Delta Encoding on i8 vectors
fn delta_encode(data : &[i8]) -> Vec<i8> {
    if data.is_empty() { return vec![]; }
    let mut encoded = Vec::with_capacity(data.len());
    encoded.push(data[0]); // Select first element as referance
    for i in 1..data.len() {
        encoded.push(data[i].wrapping_sub(data[i - 1]));
    }
    encoded
}

//Perform dynamic energy range SVD on a single channel matrix
fn encode_block_channel(matrix : &DMatrix<f32>, rank_ratio : f32) -> BlockSvd {
    // SVD calculation
    let svd = matrix.clone().svd(true, true);
    let sigma = svd.singular_values;
    let u = svd.u.as_ref().expect("Could not calculate U matrix");
    let vt = svd.v_t.as_ref().expect("Could not calculate V transpose matrix");

    
    let max_rank = sigma.len();
    let rows = matrix.nrows();
    let cols = matrix.ncols();

    //Calculate the k value
    let mut k = ((max_rank as f32) * rank_ratio).round() as usize;
    if k == 0 { k = 1; }
    if k > max_rank { k = max_rank; }

    let mut u_trimmed = Vec::with_capacity(rows * k);
    for c in 0..k {
        for r in 0..rows {
            let val = u[(r, c)].clamp(-1.0, 1.0);
            u_trimmed.push((val * 127.0).round() as i8);
        }
    }

    let mut vt_trimmed = Vec::with_capacity(k * cols);
    for c in 0..cols {
        for r in 0..k{
            let val = vt[(r, c)].clamp(-1.0, 1.0);
            vt_trimmed.push((val * 127.0).round() as i8);
        }
    }
    
    let sigma_max = if k > 0 { sigma[0] } else { 1.0 };
    let mut sigma_trimmed = Vec::with_capacity(k);
    for i in 0..k {
        let ratio = (sigma[i] / sigma_max).clamp(0.0, 1.0);
        sigma_trimmed.push((ratio * 255.0).round() as u8);
    }

    // Delta encode U and Vt
    let delta_u = delta_encode(&u_trimmed);
    let delta_vt = delta_encode(&vt_trimmed);

    BlockSvd {
        k,
        u_data : delta_u,
        sigma_data : sigma_trimmed,
        sigma_max,
        vt_data : delta_vt,
    }

}

//Referance encoder main function
pub fn encode_image(raw_rgb : &[u32], width : u32, height : u32, rank_ratio : f32) -> EncodedImage {
    let h = height as usize;
    let w = width as usize;

    let blocks_x = (w + BLOCK_SIZE - 1) / BLOCK_SIZE;
    let blocks_y = (h + BLOCK_SIZE - 1) / BLOCK_SIZE;

    let mut encoded_blocks = Vec::with_capacity(blocks_x * blocks_y);

    for by in 0..blocks_y {
        for bx in 0..blocks_x {
            //Dynamic edge calculation to prevent overflow
            let actual_rows = std::cmp::min(BLOCK_SIZE, h - by * BLOCK_SIZE);
            let actual_cols = std::cmp::min(BLOCK_SIZE, w - bx * BLOCK_SIZE);

            let mut r_mat = DMatrix::zeros(actual_rows, actual_cols);
            let mut g_mat = DMatrix::zeros(actual_rows, actual_cols);
            let mut b_mat = DMatrix::zeros(actual_rows, actual_cols);

            //Get block pixels and fill matrices
            for r in 0..actual_rows {
                let global_y = by * BLOCK_SIZE + r;
                for c in 0..actual_cols {
                    let global_x = bx * BLOCK_SIZE + c;

                    let pixel = raw_rgb[global_y * w + global_x];
                    r_mat[(r, c)] = ((pixel >> 16) & 0xFF) as f32;
                    g_mat[(r, c)] = ((pixel >> 8) & 0xFF) as f32;
                    b_mat[(r, c)] = (pixel & 0xFF) as f32;
                }
            }

            let red_svd = encode_block_channel(&r_mat, rank_ratio);
            let green_svd = encode_block_channel(&g_mat, rank_ratio);
            let blue_svd = encode_block_channel(&b_mat, rank_ratio);

            encoded_blocks.push(EncodedBlock {
                red : red_svd,
                green : green_svd,
                blue : blue_svd,
            });
        }
    }


    EncodedImage {
        width,
        height,
        block_size : BLOCK_SIZE as u32,
        blocks : encoded_blocks,
    }
}
