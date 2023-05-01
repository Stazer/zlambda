#![no_std]
#![feature(step_trait)]

////////////////////////////////////////////////////////////////////////////////////////////////////

use core::iter::Step;
use core::marker::PhantomData;
use core::mem::size_of;
use core::ops::{Add, Mul, Range};
use num_traits::Zero;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub use byteorder::*;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait ByteOrderedRead<T>
where
    T: ByteOrder,
{
    fn read(buffer: &[u8]) -> Self;
}

impl<T> ByteOrderedRead<T> for u8
where
    T: ByteOrder,
{
    fn read(buffer: &[u8]) -> Self {
        buffer[0]
    }
}

impl<T> ByteOrderedRead<T> for u16
where
    T: ByteOrder,
{
    fn read(buffer: &[u8]) -> Self {
        T::read_u16(buffer)
    }
}

impl<T> ByteOrderedRead<T> for u32
where
    T: ByteOrder,
{
    fn read(buffer: &[u8]) -> Self {
        T::read_u32(buffer)
    }
}

impl<T> ByteOrderedRead<T> for u64
where
    T: ByteOrder,
{
    fn read(buffer: &[u8]) -> Self {
        T::read_u64(buffer)
    }
}

impl<T> ByteOrderedRead<T> for i8
where
    T: ByteOrder,
{
    fn read(buffer: &[u8]) -> Self {
        buffer[0] as i8
    }
}

impl<T> ByteOrderedRead<T> for i16
where
    T: ByteOrder,
{
    fn read(buffer: &[u8]) -> Self {
        T::read_i16(buffer)
    }
}

impl<T> ByteOrderedRead<T> for i32
where
    T: ByteOrder,
{
    fn read(buffer: &[u8]) -> Self {
        T::read_i32(buffer)
    }
}

impl<T> ByteOrderedRead<T> for i64
where
    T: ByteOrder,
{
    fn read(buffer: &[u8]) -> Self {
        T::read_i64(buffer)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait ByteOrderedWrite<T>
where
    T: ByteOrder,
{
    fn write(self, buffer: &mut [u8]);
}

impl<T> ByteOrderedWrite<T> for u8
where
    T: ByteOrder,
{
    fn write(self, buffer: &mut [u8]) {
        buffer[0] = self
    }
}

impl<T> ByteOrderedWrite<T> for u16
where
    T: ByteOrder,
{
    fn write(self, buffer: &mut [u8]) {
        T::write_u16(buffer, self)
    }
}

impl<T> ByteOrderedWrite<T> for u32
where
    T: ByteOrder,
{
    fn write(self, buffer: &mut [u8]) {
        T::write_u32(buffer, self)
    }
}

impl<T> ByteOrderedWrite<T> for u64
where
    T: ByteOrder,
{
    fn write(self, buffer: &mut [u8]) {
        T::write_u64(buffer, self)
    }
}

impl<T> ByteOrderedWrite<T> for i8
where
    T: ByteOrder,
{
    fn write(self, buffer: &mut [u8]) {
        buffer[0] = self as u8
    }
}

impl<T> ByteOrderedWrite<T> for i16
where
    T: ByteOrder,
{
    fn write(self, buffer: &mut [u8]) {
        T::write_i16(buffer, self)
    }
}

impl<T> ByteOrderedWrite<T> for i32
where
    T: ByteOrder,
{
    fn write(self, buffer: &mut [u8]) {
        T::write_i32(buffer, self)
    }
}

impl<T> ByteOrderedWrite<T> for i64
where
    T: ByteOrder,
{
    fn write(self, buffer: &mut [u8]) {
        T::write_i64(buffer, self)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type MatrixValues<'a, T> = &'a [T];

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type MatrixValuesMut<'a, T> = &'a mut [T];

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
#[repr(C)]
pub struct MatrixInput<'a, D, T, B> {
    dimension_a: D,
    dimension_b: D,
    left: MatrixValues<'a, T>,
    right: MatrixValues<'a, T>,
    byte_order: PhantomData<B>,
}

impl<'a, D, T, B> MatrixInput<'a, D, T, B>
where
    D: Copy,
    B: ByteOrder,
{
    pub fn required_element_count(
        dimension_a: D,
        dimension_b: D,
    ) -> Result<usize, <usize as TryFrom<<D as Mul<D>>::Output>>::Error>
    where
        D: Mul<D>,
        usize: TryFrom<<D as Mul<D>>::Output>,
    {
        usize::try_from(dimension_a * dimension_b)
    }

    pub fn required_matrix_size(
        dimension_a: D,
        dimension_b: D,
    ) -> Result<usize, <usize as TryFrom<<D as Mul<D>>::Output>>::Error>
    where
        D: Mul<D>,
        usize: TryFrom<<D as Mul<D>>::Output>,
    {
        Ok(usize::try_from(dimension_a * dimension_b)? * size_of::<T>())
    }

    pub fn required_size(
        dimension_a: D,
        dimension_b: D,
    ) -> Result<usize, <usize as TryFrom<<D as Mul<D>>::Output>>::Error>
    where
        D: Mul<D>,
        T: 'static,
        usize: TryFrom<<D as Mul<D>>::Output>,
    {
        Ok(2 * size_of::<D>() + 2 * Self::required_matrix_size(dimension_a, dimension_b)?)
    }

    pub fn read(
        buffer: &'a mut [u8],
    ) -> Result<Self, <usize as TryFrom<<D as Mul<D>>::Output>>::Error>
    where
        D: ByteOrderedRead<B> + Mul<D> + Copy,
        T: 'static,
        B: ByteOrder,
        usize: TryFrom<<D as Mul<D>>::Output>,
    {
        let dimension_a = D::read(&buffer[0..size_of::<D>()]);
        let dimension_b = D::read(&buffer[size_of::<D>()..2 * size_of::<D>()]);

        /*let matrix_size = Self::required_matrix_size(dimension_a, dimension_b)?;

        let (_, left, _) = unsafe {
            &buffer[2 * size_of::<D>()..2 * size_of::<D>() + matrix_size].align_to::<T>()
        };
        let (_, right, _) = unsafe {
            &buffer[2 * size_of::<D>() + matrix_size..2 * size_of::<D>() + 2 * matrix_size]
                .align_to::<T>()
        };*/

        Ok(Self {
            dimension_a,
            dimension_b,
            left: &[],
            right: &[],
            //left,
            //right,
            byte_order: PhantomData,
        })
    }
}

impl<'a, D, T, B> MatrixInput<'a, D, T, B>
where
    D: Copy,
    B: ByteOrder,
{
    pub fn new(
        dimension_a: D,
        dimension_b: D,
        left: MatrixValues<'a, T>,
        right: MatrixValues<'a, T>,
    ) -> Self {
        Self {
            dimension_a,
            dimension_b,
            left,
            right,
            byte_order: PhantomData,
        }
    }

    pub fn left(&self) -> MatrixValuesView<D, MatrixValues<'a, T>> {
        MatrixValuesView {
            rows: self.dimension_a,
            columns: self.dimension_b,
            values: self.left,
        }
    }

    pub fn right(&self) -> MatrixValuesView<D, MatrixValues<'a, T>> {
        MatrixValuesView {
            rows: self.dimension_b,
            columns: self.dimension_a,
            values: self.right,
        }
    }

    pub fn dimension_a(&self) -> D {
        self.dimension_a
    }

    pub fn dimension_b(&self) -> D {
        self.dimension_b
    }

    pub fn element_count(&self) -> Result<usize, <usize as TryFrom<<D as Mul<D>>::Output>>::Error>
    where
        D: Mul<D>,
        usize: TryFrom<<D as Mul<D>>::Output>,
    {
        Self::required_element_count(self.dimension_a, self.dimension_b)
    }

    pub fn size(&self) -> Result<usize, <usize as TryFrom<<D as Mul<D>>::Output>>::Error>
    where
        D: Mul<D>,
        T: 'static,
        usize: TryFrom<<D as Mul<D>>::Output>,
    {
        Self::required_size(self.dimension_a, self.dimension_b)
    }

    pub fn write(&self, buffer: &mut [u8])
    where
        D: ByteOrderedWrite<B>,
        T: ByteOrderedWrite<B> + Copy,
    {
        self.dimension_a.write(buffer);
        self.dimension_b
            .write(&mut buffer[size_of::<D>()..2 * size_of::<D>()]);

        let mut buffer = &mut buffer[2 * size_of::<D>()..];

        for item in self.left.iter() {
            (*item).write(buffer);

            buffer = &mut buffer[size_of::<T>()..];
        }

        for item in self.right.iter() {
            (*item).write(buffer);

            buffer = &mut buffer[size_of::<T>()..];
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct MatrixValuesView<D, V> {
    rows: D,
    columns: D,
    values: V,
}

impl<'a, D, T> MatrixValuesView<D, MatrixValues<'a, T>> {
    pub fn new(rows: D, columns: D, values: MatrixValues<'a, T>) -> Self {
        Self {
            rows,
            columns,
            values,
        }
    }

    pub fn as_ptr(&self) -> *const T {
        self.values.as_ptr()
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&T>
    where
        D: Copy,
        usize: Mul<D> + From<<<usize as Mul<D>>::Output as Add<usize>>::Output>,
        <usize as Mul<D>>::Output: Add<usize>,
    {
        self.values.get(usize::from(y * self.columns + x))
    }

    pub fn multiply(
        &self,
        right: &Self,
        result: &mut MatrixValuesView<D, MatrixValuesMut<'a, T>>,
    ) -> Result<(), <usize as TryFrom<D>>::Error>
    where
        D: Step + Zero + Copy,
        T: Zero
            + Copy
            + Mul<T>
            + Add<<T as Mul<T>>::Output>
            + From<<T as Add<<T as Mul<T>>::Output>>::Output>,
        usize: TryFrom<D> + Mul<D> + From<<<usize as Mul<D>>::Output as Add<usize>>::Output>,
        <usize as Mul<D>>::Output: Add<usize>,
    {
        for i in (Range {
            start: D::zero(),
            end: self.rows,
        }) {
            for j in (Range {
                start: D::zero(),
                end: self.columns,
            }) {
                let mut value = T::zero();

                for k in (Range {
                    start: D::zero(),
                    end: self.rows,
                }) {
                    let left_value = *self.get(usize::try_from(k)?, usize::try_from(i)?).unwrap();
                    let right_value = *right.get(usize::try_from(j)?, usize::try_from(k)?).unwrap();

                    value = T::from(value + left_value * right_value);
                }

                result.set(usize::try_from(i)?, usize::try_from(j)?, value);
            }
        }

        Ok(())
    }
}

impl<'a, D, T> MatrixValuesView<D, MatrixValuesMut<'a, T>> {
    pub fn new(rows: D, columns: D, values: MatrixValuesMut<'a, T>) -> Self {
        Self {
            rows,
            columns,
            values,
        }
    }

    pub fn set(&mut self, x: usize, y: usize, value: T)
    where
        D: Copy,
        usize: Mul<D> + From<<<usize as Mul<D>>::Output as Add<usize>>::Output>,
        <usize as Mul<D>>::Output: Add<usize>,
    {
        *self
            .values
            .get_mut(usize::from(y * self.columns + x))
            .unwrap() = value
    }
}
