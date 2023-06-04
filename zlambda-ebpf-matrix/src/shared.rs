use core::marker::PhantomData;
use core::mem::size_of;
use core::ops::{Add, Mul, Range};
use core::convert::AsMut;
use core::iter::Step;
use num_traits::Zero;
use core::num::TryFromIntError;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait Access<T>
{
    fn access(&self, index: usize) -> Option<&T>;
}

impl<'a, A, T> Access<T> for &'a A
where
    A: Access<T> + 'static
{
    fn access(&self, index: usize) -> Option<&T> {
        A::access(self, index)
    }
}

impl<'a, A, T> Access<T> for &'a mut A
where
    A: Access<T> + 'static
{
    fn access(&self, index: usize) -> Option<&T> {
        A::access(self, index)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait AccessMut<T> {
    fn access_mut(&mut self, index: usize) -> Option<&mut T>;
}

impl<'a, A, T> AccessMut<T> for &'a mut A
where
    A: AccessMut<T>
{
    fn access_mut(&mut self, index: usize) -> Option<&mut T> {
        A::access_mut(self, index)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait Mutate<T> {
    fn mutate(&mut self, index: usize, value: T);
}

impl<'a, A, T> Mutate<T> for &'a mut A
where
    A: Mutate<T>
{
    fn mutate(&mut self, index: usize, value: T) {
        A::mutate(self, index, value)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Offset<T> {
    access: T,
    offset: usize,
}

impl<T> Clone for Offset<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            access: self.access.clone(),
            offset: self.offset,
        }
    }
}

impl<T> Offset<T> {
    pub fn new(access: T, offset: usize) -> Self {
        Self { access, offset }
    }

    pub fn into_inner(self) -> T {
        self.access
    }
}

impl<A, T> Access<T> for Offset<A>
where
    A: Access<T>,
{
    fn access(&self, index: usize) -> Option<&T> {
        self.access.access(self.offset + index)
    }
}

impl<A, T> AccessMut<T> for Offset<A>
where
    A: AccessMut<T>,
{
    fn access_mut(&mut self, index: usize) -> Option<&mut T> {
        self.access.access_mut(self.offset + index)
    }
}

impl<M, T> Mutate<T> for Offset<M>
where
    M: Mutate<T>
{
    fn mutate(&mut self, index: usize, value: T) {
        self.access.mutate(self.offset + index, value);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct AccessLimit<T> {
    access: T,
    limit: usize,
}

impl<T> Clone for AccessLimit<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            access: self.access.clone(),
            limit: self.limit,
        }
    }
}

impl<T> AccessLimit<T> {
    pub fn new(access: T, limit: usize) -> Self {
        Self { access, limit }
    }

    pub fn into_inner(self) -> T {
        self.access
    }
}

impl<A, T> Access<T> for AccessLimit<A>
where
    A: Access<T>,
{
    fn access(&self, index: usize) -> Option<&T> {
        if index >= self.limit {
            return None;
        }

        self.access.access(index)
    }
}

impl<A, T> AccessMut<T> for AccessLimit<A>
where
    A: AccessMut<T>,
{
    fn access_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.limit {
            return None;
        }

        self.access.access_mut(index)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait Reader {
    fn read_u8(&mut self) -> Option<u8>;
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct AccessReader<A> {
    access: A,
    offset: usize,
}

impl<A> AccessReader<A> {
    pub fn new(access: A) -> Self {
        Self { access, offset: 0 }
    }

    pub fn into_inner(self) -> A {
        self.access
    }
}

impl<A> Reader for AccessReader<A>
where
    A: Access<u8>,
{
    fn read_u8(&mut self) -> Option<u8> {
        let value = self.access.access(self.offset);
        self.offset += size_of::<u8>();

        value.copied()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait Readable: Sized {
    fn read<R>(reader: &mut R) -> Option<Self>
    where
        R: Reader;
}

impl Readable for u8 {
    fn read<R>(reader: &mut R) -> Option<Self>
    where
        R: Reader,
    {
        reader.read_u8()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type MatrixDimensionItem = u8;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct MatrixDimension {
    a: usize,
    b: usize,
}

impl MatrixDimension {
    pub fn read<'a, R>(reader: &'a mut R) -> Option<Self>
    where
        R: Reader + 'a,
    {
        let a = match u8::read(reader) {
            Some(a) => usize::from(a),
            None => return None,
        };

        let b = match u8::read(reader) {
            Some(b) => usize::from(b),
            None => return None,
        };

        Some(Self { a, b })
    }

    pub fn a(&self) -> usize {
        self.a
    }

    pub fn b(&self) -> usize {
        self.b
    }

    pub fn flip(&self) -> Self {
        Self {
            a: self.b,
            b: self.a,
        }
    }

    pub fn element_count(&self) -> usize {
        self.a * self.b
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type MatrixItem = u8;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Matrix<A, T> {
    access: A,
    dimension: MatrixDimension,
    r#type: PhantomData<T>,
}

impl<A, T> Matrix<A, T> {
    pub fn new(access: A, dimension: MatrixDimension) -> Self {
        Self {
            access,
            dimension,
            r#type: PhantomData,
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Option<MatrixItem>
    where
        A: Access<MatrixItem>,
    {
        self.access.access(self.dimension.a() * x + y).copied()
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut MatrixItem>
    where
        A: AccessMut<MatrixItem>,
    {
        self.access.access_mut(self.dimension.a() * x + y)
    }

    pub fn set(&mut self, x: usize, y: usize, value: MatrixItem)
    where
        A: Mutate<MatrixItem>,
    {
        self.access.mutate(2 * x + y, value)
    }

    pub fn multiply<A2>(&self, right: &Self, result: &mut Matrix<A2, T>)
    where
        A: Access<MatrixItem>,
        A2: Mutate<MatrixItem>,
    {
        for i in 0..2{//self.dimension.a() {
            for j in 0..2{//self.dimension.b(){
                let mut value = 0;

                for k in 0..2 {//self.dimension.a() {
                    let (left_value, right_value) = match (self.get(i, k), right.get(k, j)) {
                        (Some(left_value), Some(right_value)) => (left_value, right_value),
                        (_, _) => return,
                    };

                    value += left_value * right_value;
                }

                result.set(i, j, value);
            }
        }
    }

    pub fn into_inner(self) -> A {
        self.access
    }
}
