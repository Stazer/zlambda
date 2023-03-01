use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::str::FromStr;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct TaggedType<T1, T2>(T1, PhantomData<T2>);

impl<T1, T2> Clone for TaggedType<T1, T2>
where
    T1: Clone,
{
    fn clone(&self) -> Self {
        Self::new(self.0.clone())
    }
}

impl<T1, T2> Copy for TaggedType<T1, T2> where T1: Copy {}

impl<T1, T2> Debug for TaggedType<T1, T2>
where
    T1: Debug,
{
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        Debug::fmt(&self.0, formatter)
    }
}

impl<T1, T2> Default for TaggedType<T1, T2>
where
    T1: Default,
{
    fn default() -> Self {
        Self::new(T1::default())
    }
}

impl<'de, T1, T2> Deserialize<'de> for TaggedType<T1, T2>
where
    T1: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self::new(T1::deserialize(deserializer)?))
    }
}

impl<T1, T2> Display for TaggedType<T1, T2>
where
    T1: Display,
{
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.0, formatter)
    }
}

impl<T1, T2> Eq for TaggedType<T1, T2> where T1: Eq {}

impl<T1, T2> const From<T1> for TaggedType<T1, T2> {
    fn from(inner: T1) -> Self {
        Self::new(inner)
    }
}

impl<T2> const From<TaggedType<usize, T2>> for usize {
    fn from(r#type: TaggedType<usize, T2>) -> Self {
        r#type.0
    }
}

impl<T1, T2> FromStr for TaggedType<T1, T2>
where
    T1: FromStr,
{
    type Err = <T1 as FromStr>::Err;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(<T1 as FromStr>::from_str(string)?))
    }
}

impl<T1, T2> Hash for TaggedType<T1, T2>
where
    T1: Hash,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.0.hash(state)
    }

    // TODO
    /*fn hash_slice<H>(data: &[Self], state: &mut H)
    where
        H: Hasher,
        Self: Sized,
    {
        <T1 as Hash>::hash_slice(&data, state)
    }*/
}

impl<T1, T2> Ord for TaggedType<T1, T2>
where
    T1: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        <T1 as Ord>::cmp(&self.0, &other.0)
    }

    fn max(self, other: Self) -> Self {
        Self::new(<T1 as Ord>::max(self.0, other.0))
    }

    fn min(self, other: Self) -> Self {
        Self::new(<T1 as Ord>::min(self.0, other.0))
    }

    fn clamp(self, min: Self, max: Self) -> Self {
        Self::new(<T1 as Ord>::clamp(self.0, min.0, max.0))
    }
}

impl<T1, T2> PartialEq for TaggedType<T1, T2>
where
    T1: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T1, T2> PartialOrd for TaggedType<T1, T2>
where
    T1: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        <T1 as PartialOrd>::partial_cmp(&self.0, &other.0)
    }

    fn lt(&self, other: &Self) -> bool {
        <T1 as PartialOrd>::lt(&self.0, &other.0)
    }

    fn le(&self, other: &Self) -> bool {
        <T1 as PartialOrd>::le(&self.0, &other.0)
    }

    fn gt(&self, other: &Self) -> bool {
        <T1 as PartialOrd>::gt(&self.0, &other.0)
    }

    fn ge(&self, other: &Self) -> bool {
        <T1 as PartialOrd>::ge(&self.0, &other.0)
    }
}

impl<T1, T2> Serialize for TaggedType<T1, T2>
where
    T1: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        T1::serialize(&self.0, serializer)
    }
}

impl<T1, T2> TaggedType<T1, T2> {
    const fn new(inner: T1) -> Self {
        Self(inner, PhantomData::<T2>)
    }
}
