use std::slice::{from_raw_parts, from_raw_parts_mut};

////////////////////////////////////////////////////////////////////////////////////////////////////

const MATRIX_SIZE: usize = 128;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[no_mangle]
pub extern "C" fn main(data: *mut u8) {
    let left = unsafe { from_raw_parts(data, MATRIX_SIZE * MATRIX_SIZE) };
    let right = unsafe {
        from_raw_parts(
            data.add(MATRIX_SIZE * MATRIX_SIZE),
            MATRIX_SIZE * MATRIX_SIZE,
        )
    };
    let result = unsafe {
        from_raw_parts_mut(
            data.add(MATRIX_SIZE * MATRIX_SIZE * 2),
            MATRIX_SIZE * MATRIX_SIZE,
        )
    };

    for i in 0..MATRIX_SIZE {
        for j in 0..MATRIX_SIZE {
            let mut value = 0;

            for k in 0..MATRIX_SIZE {
                let (left_value, right_value) = match (
                    left.get(i * MATRIX_SIZE + k),
                    right.get(k * MATRIX_SIZE + j),
                ) {
                    (Some(left_value), Some(right_value)) => (left_value, right_value),
                    (_, _) => return,
                };

                value += left_value * right_value;
            }

            if let Some(old_value) = result.get_mut(i * MATRIX_SIZE + j) {
                *old_value = value;
            }
        }
    }
}
