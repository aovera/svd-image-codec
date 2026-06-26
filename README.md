# sdv (SVD Image Codec) • v0.2.0

`sdv` is an experimental image codec built from scratch in Rust, leveraging one of the most powerful tools in Linear Algebra: **Singular Value Decomposition (SVD)**. 

Unlike naive SVD compression methods that process an entire image as a single massive matrix, `sdv` utilizes **Block-based Decomposition** and **Linear Rank Truncation**, mirroring the architecture of modern image and video codecs. With Version 2.0, the codec integrates a robust **Lossless Entropy Pipeline**, combining integer quantization and DEFLATE algorithms to achieve massive file size reductions without compromising visual fidelity.

## Features

* **Block-based Encoding:** Segments the image into independent 16x16 pixel blocks to prevent global blurriness and maintain sharp local details.
* **Linear Rank Truncation:** Replaces volatile energy-based thresholds with predictable, user-controlled rank ratio (k) throttling.
* **Deep Quantization Pipeline:** Maps floating-point matrix elements into 8-bit integers
* **Lossless Entropy Coding:** Utilizes Delta Encoding (DPCM) combined with high-ratio DEFLATE (Zlib) compression to eliminate spatial redundancy and push file sizes to their theoretical minimum.
* **Direct Framebuffer View:** Renders compressed `.svd` files directly onto the native OS window manager via `minifb`, bypassing the need for intermediate format conversions.

## Installation

Ensure you have Rust and Cargo installed on your system.

```git clone https://github.com/aovera/sdv-image-codec```

```cd sdv-image-codec```

```cargo build --release```

Usage

You can run the compiled binary located at target/release/sdv (or sdv.exe on Windows) directly, or use cargo run -- followed by the arguments.
1. Compressing an Image (Encode)

Compress any .png or .jpg image into the .svd format by specifying your desired rank retention ratio:
Bash

# Encode using the default quality (retains 95% of components)
cargo run -- encode input.png output.svd

# Selective compression - retains only 30% of components
cargo run -- encode -k 30 input.png output.svd

2. Displaying an SVD File (View)

Open and render the compressed .svd file directly inside a live graphical frame buffer:

cargo run -- view output.svd

Press ESC at any time to close the viewer window.
Technical Architecture

    Channel Separation: The incoming RGB image is unpacked into 3 independent f32 matrices (Red, Green, and Blue) to prevent color cross-talk and maintain linear algebraic integrity.

    Localization: The image grid is split into discrete 16x16 blocks. Residual blocks on the bottom and right edges are calculated dynamically to prevent memory padding artifacts.

    Matrix Decomposition: Each block matrix undergoes SVD to compute its singular vectors and values: A = U * Σ * V^T.

    Rank Truncation: Based on the user-defined -k percentage, the codec isolates the top k rows and columns containing the highest spatial variance, discarding the rest.

    Quantization & DPCM: The remaining f32 spatial vectors are clamped and scaled into single-byte i8 ranges. A Delta Encoding (Differential Pulse-Code Modulation) pass flattens spatial variance, generating long streaks of zeros and low-value integers.

    Entropy Compression: The quantized blocks are serialized via bincode and piped through a maximum-compression DEFLATE (flate2) stream to finalize the .svd file.

Roadmap (Version 0.3.0 Specifications)

Version 0.2.0 successfully implements the mathematical and entropy baseline. The next major milestone will focus on exploiting the Human Visual System (HVS):

    [ ] Color Space Transformation: Migrate from standard RGB matrices to the YCbCr (Luma, Chroma Blue, Chroma Red) color space.

    [ ] Chroma Subsampling: Apply aggressive, asymmetric rank truncation to the Cb and Cr channels while preserving Luma (Y) fidelity, drastically reducing file sizes with zero perceptual quality loss to the human eye.
