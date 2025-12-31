# SIMD and Vectorization in Rust

Let me provide you with a comprehensive guide to SIMD (Single Instruction, Multiple Data) and vectorization in Rust, complete with practical examples.I've created a comprehensive guide to SIMD and vectorization in Rust with practical examples. Let me explain the key concepts:

## What is SIMD?

**SIMD (Single Instruction, Multiple Data)** is a parallel computing technique where one instruction processes multiple data elements simultaneously. Instead of adding two numbers at a time, you can add 4, 8, or even 16 numbers in a single CPU instruction.

## Key Approaches in Rust

### 1. **Platform-Specific Intrinsics**
Using architecture-specific instructions (like AVX2, SSE, or NEON):
- Fastest performance
- Most control
- Requires `unsafe` code
- Not portable across architectures
- Example: `_mm256_add_ps` adds 8 floats at once on x86_64

### 2. **Portable SIMD** (std::simd)
The future of SIMD in Rust (currently nightly):
- Write once, run on any architecture
- Safe abstractions
- Compiler picks the best instructions
- Uses types like `f32x8` for 8 floats at once

### 3. **Auto-Vectorization**
Let the compiler do the work:
- Write clean iterator code
- Compiler automatically generates SIMD
- Works with stable Rust
- Less predictable than explicit SIMD

## Cache-Friendly Patterns

Performance isn't just about SIMD—memory access patterns matter enormously:

**Bad practices:**
- Random memory access
- Column-major iteration of row-major data
- Array of Structures (AoS) when processing single fields

**Good practices:**
- Sequential memory access
- Blocked/tiled algorithms
- Structure of Arrays (SoA) layout
- Proper alignment (16/32 bytes)

## Real-World Benefits

SIMD can provide **2-8x speedups** for operations like:
- Image/video processing
- Audio processing
- Physics simulations
- Machine learning inference
- Data compression
- Cryptography

## Best Practices

1. **Start simple**: Use iterators and let the compiler vectorize
2. **Profile first**: Measure before optimizing
3. **Compile with target-cpu=native**: `RUSTFLAGS='-C target-cpu=native' cargo build --release`
4. **Check assembly**: Use `cargo rustc -- --emit asm` to verify SIMD generation
5. **Use crates**: Consider `packed_simd_2`, `wide`, or `faster` for easier SIMD
6. **Keep hot data together**: Use SoA layout for better cache utilization

The artifact includes 8 detailed examples showing these concepts in action, from basic SIMD operations to cache-friendly algorithms and real-world image processing.

```rust
// ============================================================================
// SIMD and Vectorization in Rust
// ============================================================================

// 1. BASIC CONCEPT: Processing multiple data elements with one instruction
// Without SIMD: Process 4 additions one at a time (4 operations)
// With SIMD: Process 4 additions simultaneously (1 operation)

// ============================================================================
// EXAMPLE 1: Platform-Specific SIMD (x86/x86_64)
// ============================================================================

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// Add two arrays using platform-specific SIMD intrinsics
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn add_arrays_simd_avx2(a: &[f32], b: &[f32], result: &mut [f32]) {
    assert_eq!(a.len(), b.len());
    assert_eq!(a.len(), result.len());
    
    let mut i = 0;
    // Process 8 floats at a time using AVX2 (256-bit registers)
    while i + 8 <= a.len() {
        // Load 8 floats from each array
        let va = _mm256_loadu_ps(a.as_ptr().add(i));
        let vb = _mm256_loadu_ps(b.as_ptr().add(i));
        
        // Add all 8 pairs simultaneously
        let vresult = _mm256_add_ps(va, vb);
        
        // Store result
        _mm256_storeu_ps(result.as_mut_ptr().add(i), vresult);
        
        i += 8;
    }
    
    // Handle remaining elements
    for j in i..a.len() {
        result[j] = a[j] + b[j];
    }
}

// ============================================================================
// EXAMPLE 2: Portable SIMD (std::simd - Nightly Rust)
// ============================================================================

// Note: Requires #![feature(portable_simd)] on nightly Rust
// Uncomment the following for nightly:

/*
use std::simd::prelude::*;

/// Add two arrays using portable SIMD (works across platforms)
fn add_arrays_portable_simd(a: &[f32], b: &[f32]) -> Vec<f32> {
    assert_eq!(a.len(), b.len());
    
    let mut result = Vec::with_capacity(a.len());
    
    // Process in chunks of 8 (or whatever SIMD width is optimal)
    let chunks = a.len() / 8;
    
    for i in 0..chunks {
        let offset = i * 8;
        
        // Load SIMD vectors
        let va = f32x8::from_slice(&a[offset..offset + 8]);
        let vb = f32x8::from_slice(&b[offset..offset + 8]);
        
        // Perform SIMD addition
        let vresult = va + vb;
        
        // Store result
        result.extend_from_slice(vresult.as_array());
    }
    
    // Handle remainder
    for i in (chunks * 8)..a.len() {
        result.push(a[i] + b[i]);
    }
    
    result
}

/// Dot product using portable SIMD
fn dot_product_simd(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());
    
    let mut sum = f32x8::splat(0.0);
    let chunks = a.len() / 8;
    
    for i in 0..chunks {
        let offset = i * 8;
        let va = f32x8::from_slice(&a[offset..offset + 8]);
        let vb = f32x8::from_slice(&b[offset..offset + 8]);
        sum += va * vb;
    }
    
    // Sum all lanes
    let mut result = sum.reduce_sum();
    
    // Add remainder
    for i in (chunks * 8)..a.len() {
        result += a[i] * b[i];
    }
    
    result
}
*/

// ============================================================================
// EXAMPLE 3: Auto-Vectorization with Iterator Patterns
// ============================================================================

/// Scalar version - baseline
fn sum_of_squares_scalar(data: &[f32]) -> f32 {
    let mut sum = 0.0;
    for &x in data {
        sum += x * x;
    }
    sum
}

/// Iterator version - helps compiler auto-vectorize
fn sum_of_squares_iter(data: &[f32]) -> f32 {
    data.iter().map(|&x| x * x).sum()
}

/// Explicit chunking to encourage vectorization
fn sum_of_squares_chunks(data: &[f32]) -> f32 {
    data.chunks_exact(4)
        .map(|chunk| {
            chunk.iter().map(|&x| x * x).sum::<f32>()
        })
        .sum::<f32>()
        + data.chunks_exact(4).remainder()
            .iter()
            .map(|&x| x * x)
            .sum::<f32>()
}

// ============================================================================
// EXAMPLE 4: Cache-Friendly Code Patterns
// ============================================================================

/// BAD: Poor cache locality (column-major access of row-major data)
fn transpose_cache_unfriendly(matrix: &[f32], rows: usize, cols: usize) -> Vec<f32> {
    let mut result = vec![0.0; rows * cols];
    
    // Accessing columns in row-major layout causes cache misses
    for col in 0..cols {
        for row in 0..rows {
            result[col * rows + row] = matrix[row * cols + col];
        }
    }
    
    result
}

/// GOOD: Better cache locality (blocked/tiled access)
fn transpose_cache_friendly(matrix: &[f32], rows: usize, cols: usize) -> Vec<f32> {
    let mut result = vec![0.0; rows * cols];
    const BLOCK_SIZE: usize = 16; // Fits in cache
    
    // Process in blocks to maintain cache locality
    for row_block in (0..rows).step_by(BLOCK_SIZE) {
        for col_block in (0..cols).step_by(BLOCK_SIZE) {
            let row_end = (row_block + BLOCK_SIZE).min(rows);
            let col_end = (col_block + BLOCK_SIZE).min(cols);
            
            for row in row_block..row_end {
                for col in col_block..col_end {
                    result[col * rows + row] = matrix[row * cols + col];
                }
            }
        }
    }
    
    result
}

/// Cache-friendly matrix multiplication (blocked)
fn matmul_cache_friendly(a: &[f32], b: &[f32], n: usize) -> Vec<f32> {
    let mut c = vec![0.0; n * n];
    const BLOCK: usize = 32;
    
    for i_block in (0..n).step_by(BLOCK) {
        for j_block in (0..n).step_by(BLOCK) {
            for k_block in (0..n).step_by(BLOCK) {
                // Process block
                for i in i_block..(i_block + BLOCK).min(n) {
                    for j in j_block..(j_block + BLOCK).min(n) {
                        let mut sum = c[i * n + j];
                        for k in k_block..(k_block + BLOCK).min(n) {
                            sum += a[i * n + k] * b[k * n + j];
                        }
                        c[i * n + j] = sum;
                    }
                }
            }
        }
    }
    
    c
}

// ============================================================================
// EXAMPLE 5: Structure of Arrays (SoA) vs Array of Structures (AoS)
// ============================================================================

/// Array of Structures (AoS) - less cache-friendly
#[derive(Clone)]
struct ParticleAoS {
    x: f32,
    y: f32,
    z: f32,
    vx: f32,
    vy: f32,
    vz: f32,
}

fn update_particles_aos(particles: &mut [ParticleAoS], dt: f32) {
    // Accessing x, then vx means jumping around in memory
    for p in particles.iter_mut() {
        p.x += p.vx * dt;
        p.y += p.vy * dt;
        p.z += p.vz * dt;
    }
}

/// Structure of Arrays (SoA) - more cache-friendly and SIMD-friendly
struct ParticlesSoA {
    x: Vec<f32>,
    y: Vec<f32>,
    z: Vec<f32>,
    vx: Vec<f32>,
    vy: Vec<f32>,
    vz: Vec<f32>,
}

fn update_particles_soa(particles: &mut ParticlesSoA, dt: f32) {
    // Sequential access, perfect for SIMD and cache
    for i in 0..particles.x.len() {
        particles.x[i] += particles.vx[i] * dt;
        particles.y[i] += particles.vy[i] * dt;
        particles.z[i] += particles.vz[i] * dt;
    }
}

// ============================================================================
// EXAMPLE 6: Alignment for SIMD Performance
// ============================================================================

use std::alloc::{alloc, dealloc, Layout};

/// Allocate aligned memory for better SIMD performance
struct AlignedVec<T> {
    ptr: *mut T,
    len: usize,
    capacity: usize,
}

impl<T> AlignedVec<T> {
    /// Create with 32-byte alignment (for AVX)
    fn new_aligned(capacity: usize) -> Self {
        let layout = Layout::from_size_align(
            capacity * std::mem::size_of::<T>(),
            32, // 32-byte alignment for AVX
        ).unwrap();
        
        let ptr = unsafe { alloc(layout) as *mut T };
        
        Self {
            ptr,
            len: 0,
            capacity,
        }
    }
}

impl<T> Drop for AlignedVec<T> {
    fn drop(&mut self) {
        unsafe {
            let layout = Layout::from_size_align_unchecked(
                self.capacity * std::mem::size_of::<T>(),
                32,
            );
            dealloc(self.ptr as *mut u8, layout);
        }
    }
}

// ============================================================================
// EXAMPLE 7: Practical Real-World Example - Image Processing
// ============================================================================

/// Apply brightness adjustment to image (scalar version)
fn adjust_brightness_scalar(pixels: &mut [u8], adjustment: i16) {
    for pixel in pixels.iter_mut() {
        let new_val = (*pixel as i16 + adjustment).clamp(0, 255);
        *pixel = new_val as u8;
    }
}

/// Apply brightness adjustment (SIMD-friendly version)
fn adjust_brightness_vectorized(pixels: &mut [u8], adjustment: i16) {
    // Process in chunks that compiler can vectorize
    pixels.iter_mut().for_each(|pixel| {
        *pixel = (*pixel as i16 + adjustment).clamp(0, 255) as u8;
    });
}

// ============================================================================
// EXAMPLE 8: Using third-party crates for easier SIMD
// ============================================================================

// Common crates:
// - `packed_simd_2` - SIMD types and operations
// - `simdeez` - Easy SIMD abstraction
// - `wide` - Safe SIMD types
// - `faster` - Iterator-based SIMD

/*
// Example with `wide` crate:
use wide::f32x8;

fn process_with_wide(data: &[f32]) -> Vec<f32> {
    let mut result = Vec::with_capacity(data.len());
    
    for chunk in data.chunks_exact(8) {
        let simd = f32x8::from(chunk);
        let processed = simd * f32x8::splat(2.0) + f32x8::splat(1.0);
        result.extend_from_slice(processed.as_array_ref());
    }
    
    // Handle remainder
    for &x in data.chunks_exact(8).remainder() {
        result.push(x * 2.0 + 1.0);
    }
    
    result
}
*/

// ============================================================================
// PERFORMANCE TIPS
// ============================================================================

// 1. Use `#[inline]` or `#[inline(always)]` for small SIMD functions
// 2. Avoid branches inside SIMD loops when possible
// 3. Keep data aligned (16/32 bytes) for best performance
// 4. Process data in contiguous chunks
// 5. Use Structure of Arrays (SoA) over Array of Structures (AoS)
// 6. Let the compiler auto-vectorize when possible with iterator patterns
// 7. Profile before and after optimizations
// 8. Use `cargo rustc -- --emit asm` to inspect generated assembly

fn main() {
    println!("SIMD and Vectorization Examples in Rust");
    println!("========================================\n");
    
    // Example: Compare different approaches
    let data: Vec<f32> = (0..1000).map(|x| x as f32).collect();
    
    let sum1 = sum_of_squares_scalar(&data);
    let sum2 = sum_of_squares_iter(&data);
    
    println!("Sum of squares (scalar): {}", sum1);
    println!("Sum of squares (iterator): {}", sum2);
    
    // Cache-friendly example
    let matrix: Vec<f32> = (0..100).map(|x| x as f32).collect();
    let transposed = transpose_cache_friendly(&matrix, 10, 10);
    println!("\nTransposed first few elements: {:?}", &transposed[0..5]);
    
    // SoA vs AoS
    let mut particles = ParticlesSoA {
        x: vec![1.0; 1000],
        y: vec![2.0; 1000],
        z: vec![3.0; 1000],
        vx: vec![0.1; 1000],
        vy: vec![0.2; 1000],
        vz: vec![0.3; 1000],
    };
    
    update_particles_soa(&mut particles, 0.016);
    println!("\nUpdated first particle position: ({}, {}, {})",
             particles.x[0], particles.y[0], particles.z[0]);
    
    println!("\n✓ Compile with `RUSTFLAGS='-C target-cpu=native'` for best SIMD performance");
}
```