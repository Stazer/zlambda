use core::marker::PhantomData;
use core::mem::size_of;
use core::ops::{Add, Mul};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait Access<T> {
    fn access(&self, index: usize) -> Option<&T>;
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait AccessMut<T> {
    fn access_mut(&mut self, index: usize) -> Option<&mut T>;
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct AccessOffset<T> {
    access: T,
    offset: usize,
}

impl<T> Clone for AccessOffset<T>
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

impl<T> AccessOffset<T> {
    pub fn new(access: T, offset: usize) -> Self {
        Self { access, offset }
    }

    pub fn into_inner(self) -> T {
        self.access
    }
}

impl<A, T> Access<T> for AccessOffset<A>
where
    A: Access<T>,
{
    fn access(&self, index: usize) -> Option<&T> {
        self.access.access(self.offset + index)
    }
}

impl<A, T> AccessMut<T> for AccessOffset<A>
where
    A: AccessMut<T>,
{
    fn access_mut(&mut self, index: usize) -> Option<&mut T> {
        self.access.access_mut(self.offset + index)
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

pub struct AccessIterator<A, T> {
    access: A,
    offset: usize,
    r#type: PhantomData<T>,
}

impl<A, T> AccessIterator<A, T> {
    pub fn new(access: A) -> Self {
        Self {
            access,
            offset: 0,
            r#type: PhantomData,
        }
    }
}

impl<'a, A, T> Iterator for AccessIterator<&'a A, T>
where
    A: Access<T>,
    T: 'a,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let item = match self.access.access(self.offset) {
            Some(item) => item,
            None => return None,
        };

        self.offset += 1;

        Some(item)
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

pub struct MatrixDimension<T> {
    a: T,
    b: T,
}

impl<T> MatrixDimension<T> {
    pub fn read<R>(reader: &mut R) -> Option<Self>
    where
        R: Reader,
        T: Readable,
    {
        let a = match T::read(reader) {
            Some(a) => a,
            None => return None,
        };

        let b = match T::read(reader) {
            Some(b) => b,
            None => return None,
        };

        Some(Self { a, b })
    }

    pub fn a(&self) -> &T {
        &self.a
    }

    pub fn b(&self) -> &T {
        &self.b
    }

    pub fn flip(&self) -> Self
    where
        T: Clone,
    {
        Self {
            a: self.b.clone(),
            b: self.a.clone(),
        }
    }

    pub fn element_count(&self) -> Result<usize, <T as TryInto<usize>>::Error>
    where
        T: TryInto<usize> + Copy,
    {
        Ok(self.a.try_into()? * self.b.try_into()?)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct MatrixAccess<A, D, T> {
    access: A,
    dimension: MatrixDimension<D>,
    r#type: PhantomData<T>,
}

impl<A, D, T> MatrixAccess<A, D, T> {
    pub fn new(access: A, dimension: MatrixDimension<D>) -> Self {
        Self {
            access,
            dimension,
            r#type: PhantomData,
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Result<Option<T>, <D as TryInto<usize>>::Error>
    where
        A: Access<T>,
        D: Copy + TryInto<usize>,
        T: Copy,
    {
        Ok(self.access.access((*self.dimension.a()).try_into()? * x + y))
    }

    /*pub fn get_mut(
        &mut self,
        x: usize,
        y: usize,
    ) -> Result<Option<&mut T>, <&D as TryInto<usize>>::Error>
    where
        A: AccessMut<T>,
        D: TryInto<usize> + Copy,
    {
        Ok(self
            .access
            .access_mut(self.dimension.a().try_into()? * x + y))
    }*/

    pub fn into_inner(self) -> A {
        self.access
    }
}
