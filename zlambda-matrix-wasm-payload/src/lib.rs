use std::slice::{from_raw_parts, from_raw_parts_mut};
use zlambda_matrix::{MATRIX_SIZE, MATRIX_DIMENSION_SIZE};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[no_mangle]
pub extern "C" fn main(data: *mut u8) {
    let left = unsafe { from_raw_parts(data, MATRIX_SIZE) };
    let right = unsafe {
        from_raw_parts(
            data.add(MATRIX_SIZE),
            MATRIX_SIZE,
        )
    };
    let result = unsafe {
        from_raw_parts_mut(
            data.add(MATRIX_SIZE * 2),
            MATRIX_SIZE,
        )
    };

    for i in 0..MATRIX_DIMENSION_SIZE {
        for j in 0..MATRIX_DIMENSION_SIZE {
            let mut value = 0;

            for k in 0..MATRIX_DIMENSION_SIZE {
                let (left_value, right_value) = (
                    left.get(i * MATRIX_DIMENSION_SIZE + k).unwrap(),
                    right.get(k * MATRIX_DIMENSION_SIZE + j).unwrap(),
                );

                value += (*left_value as usize) * (*right_value as usize);
            }

            *result.get_mut(i * MATRIX_DIMENSION_SIZE + j).unwrap() = value as u8;
        }
    }
}
