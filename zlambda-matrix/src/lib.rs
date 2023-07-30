#![no_std]

////////////////////////////////////////////////////////////////////////////////////////////////////

use core::mem::size_of;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub const MATRIX_DIMENSION_SIZE: usize = 128;
pub const MATRIX_ELEMENT_COUNT: usize = MATRIX_DIMENSION_SIZE * MATRIX_DIMENSION_SIZE;
pub const MATRIX_ELEMENT_SIZE: usize = size_of::<u8>();
pub const MATRIX_SIZE: usize = MATRIX_ELEMENT_COUNT * MATRIX_ELEMENT_SIZE;
