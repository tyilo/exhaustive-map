use crate::Finite;

macro_rules! impl_deref {
    ($type:path) => {
        impl<T: Finite> Finite for $type {
            type INHABITANTS = T::INHABITANTS;

            fn to_usize(&self) -> usize {
                (**self).to_usize()
            }

            fn from_usize(i: usize) -> Option<Self> {
                Some(T::from_usize(i)?.into())
            }
        }
    };
}

impl_deref!(alloc::boxed::Box<T>);
impl_deref!(alloc::rc::Rc<T>);
impl_deref!(alloc::sync::Arc<T>);

impl<T: Finite + Clone> Finite for alloc::borrow::Cow<'_, T> {
    type INHABITANTS = T::INHABITANTS;

    fn to_usize(&self) -> usize {
        (**self).to_usize()
    }

    fn from_usize(i: usize) -> Option<Self> {
        Some(Self::Owned(T::from_usize(i)?))
    }
}

#[cfg(all(test, feature = "std"))]
mod test {
    use super::super::test_utils::test_all;

    #[test]
    fn test_cow_arr() {
        test_all::<alloc::borrow::Cow<[bool; 2]>>(4);
    }
}
