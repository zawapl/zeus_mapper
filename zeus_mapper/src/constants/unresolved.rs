use crate::constants::data_constant::DataConstant;
use std::ops::Deref;

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Unresolved<T: DataConstant>(T::Primitive);

impl<T: DataConstant> Unresolved<T> {
    pub fn new(value: T::Primitive) -> Self {
        return Self(value);
    }

    pub fn of(value: T) -> Self {
        return Self::new(value.value());
    }

    pub fn try_resolve(&self) -> Option<T> {
        return T::try_resolve(&self.0);
    }
}

impl<T: DataConstant> Deref for Unresolved<T> {
    type Target = T::Primitive;

    fn deref(&self) -> &Self::Target {
        return &self.0;
    }
}

impl<T: DataConstant> From<T> for Unresolved<T> {
    fn from(value: T) -> Self {
        return Unresolved::new(value.value());
    }
}

#[cfg(test)]
mod tests {
    use crate::constants::data_constant::data_constants;
    use crate::prelude::Unresolved;

    data_constants!(Test<u16> {
        One = 1,
        Two = 2,
    });

    #[test]
    fn test_enum() {
        assert_eq!(Unresolved::<Test>::new(1).try_resolve(), Some(Test::One));
        assert_eq!(Unresolved::<Test>::new(2).try_resolve(), Some(Test::Two));

        assert_eq!(Unresolved::<Test>::new(0).try_resolve(), None);
        assert_eq!(Unresolved::<Test>::new(3).try_resolve(), None);
    }
}
