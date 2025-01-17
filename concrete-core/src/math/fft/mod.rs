//! Fourier transform for polynomials.
//!
//! This module provides the tools to perform a fast product of two polynomials, reduced modulo
//! $X^N+1$, using the fast fourier transform.

#[cfg(test)]
mod tests;

mod twiddles;
use twiddles::*;

mod polynomial;
pub use polynomial::*;

mod transform;
pub use transform::*;

/// A complex number encoded over two `f64`.
pub type Complex64 = fftw::types::c64;
