use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::{from_reader, to_writer_pretty};
use std::error::Error;
use std::io::{stdin, stdout, Read, Write};
use std::marker::PhantomData;
use std::ops::Mul;
use std::ptr::copy_nonoverlapping;
use zlambda_matrix::{ByteOrder, MatrixInput, NativeEndian};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
pub struct MatrixCliInput<D, T, B> {
    dimension_a: D,
    dimension_b: D,
    left: Vec<T>,
    right: Vec<T>,
    #[serde(skip, default)]
    byte_order: PhantomData<B>,
}

impl<'a, D, T, B> From<&'a MatrixCliInput<D, T, B>> for MatrixInput<'a, D, T, B>
where
    D: Copy,
    B: ByteOrder,
{
    fn from(input: &'a MatrixCliInput<D, T, B>) -> Self {
        Self::new(
            input.dimension_a,
            input.dimension_b,
            &input.left,
            &input.right,
        )
    }
}

impl<'a, D, T, B> TryFrom<MatrixInput<'a, D, T, B>> for MatrixCliInput<D, T, B>
where
    D: Mul<D> + Copy,
    B: ByteOrder,
    T: Default,
    usize: TryFrom<<D as Mul<D>>::Output>,
{
    type Error = <usize as TryFrom<<D as Mul<D>>::Output>>::Error;

    fn try_from(input: MatrixInput<'a, D, T, B>) -> Result<Self, Self::Error> {
        let element_count = input.element_count()?;

        let mut left = Vec::default();
        left.resize_with(element_count, || T::default());

        let mut right = Vec::default();
        right.resize_with(element_count, || T::default());

        unsafe {
            copy_nonoverlapping(input.left().as_ptr(), left.as_mut_ptr(), element_count);
            copy_nonoverlapping(input.right().as_ptr(), right.as_mut_ptr(), element_count);
        }

        Ok(Self {
            dimension_a: input.dimension_a(),
            dimension_b: input.dimension_b(),
            left,
            right,
            byte_order: PhantomData,
        })
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
enum Command {
    Serialize,
    Deserialize,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

fn main() -> Result<(), Box<dyn Error>> {
    match Command::parse() {
        Command::Serialize => {
            let cli_input = from_reader::<_, MatrixCliInput<u8, u8, NativeEndian>>(stdin())?;
            let input = MatrixInput::from(&cli_input);

            let mut buffer = Vec::<u8>::new();
            buffer.resize_with(input.size()?, || 0);

            input.write(buffer.as_mut_slice());

            stdout().write_all(&buffer)?;
        }
        Command::Deserialize => {
            let mut buffer = Vec::<u8>::new();

            let mut handle = stdin();
            handle.read_to_end(&mut buffer)?;

            let input: MatrixInput<'_, u8, u8, NativeEndian> =
                MatrixInput::read(buffer.as_mut_slice())?;

            to_writer_pretty(&stdout(), &MatrixCliInput::try_from(input)?)?;
        }
    }

    Ok(())
}
