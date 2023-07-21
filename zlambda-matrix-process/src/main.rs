use bytes::BytesMut;
use zlambda_matrix::{
    MATRIX_DIMENSION_SIZE,
    MATRIX_ELEMENT_COUNT,
};
use std::slice::{from_raw_parts, from_raw_parts_mut};
use std::io::{Read, Write, stdin, stdout};
use std::error::Error;

////////////////////////////////////////////////////////////////////////////////////////////////////

fn main() -> Result<(), Box<dyn Error>> {
    let mut stdin = stdin();

    let mut data = BytesMut::zeroed(MATRIX_ELEMENT_COUNT * 3);
    let mut written = 0;

    while written < data.len() - MATRIX_ELEMENT_COUNT {
        written += stdin.read(unsafe {
            from_raw_parts_mut(data.as_mut_ptr().add(written), data.len() - written)
        })?;
    }

    let left = unsafe { from_raw_parts(data.as_mut_ptr(), MATRIX_DIMENSION_SIZE * MATRIX_DIMENSION_SIZE) };
    let right = unsafe {
        from_raw_parts(
            data.as_mut_ptr().add(MATRIX_DIMENSION_SIZE * MATRIX_DIMENSION_SIZE),
            MATRIX_DIMENSION_SIZE * MATRIX_DIMENSION_SIZE,
        )
    };
    let result = unsafe {
        from_raw_parts_mut(
            data.as_mut_ptr().add(MATRIX_DIMENSION_SIZE * MATRIX_DIMENSION_SIZE * 2),
            MATRIX_DIMENSION_SIZE * MATRIX_DIMENSION_SIZE,
        )
    };

    for i in 0..MATRIX_DIMENSION_SIZE {
        for j in 0..MATRIX_DIMENSION_SIZE {
            let mut value: usize = 0;

            for k in 0..MATRIX_DIMENSION_SIZE {
                let (left_value, right_value) = match (
                    left.get(i * MATRIX_DIMENSION_SIZE + k),
                    right.get(k * MATRIX_DIMENSION_SIZE + j),
                ) {
                    (Some(left_value), Some(right_value)) => (left_value, right_value),
                    (_, _) => return Ok(()),
                };

                value += (*left_value as usize) * (*right_value as usize);
            }

            if let Some(old_value) = result.get_mut(i * MATRIX_DIMENSION_SIZE + j) {
                *old_value = value as u8;
            }
        }
    }

    let mut stdout = stdout();
    stdout.write_all(&data)?;

    Ok(())
}
