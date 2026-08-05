pub trait DataConstant
where
    Self: Sized,
{
    type Primitive;

    fn try_resolve(value: &Self::Primitive) -> Option<Self>;

    fn value(&self) -> Self::Primitive;

    fn values() -> &'static [Self];
}

macro_rules! data_constants {
    ( $enum_name:tt<$primitive_type:tt> { $($(#[$variant_attr:meta])* $variant:ident = $value:expr,)+} ) => {

        #[derive(PartialEq, Debug, Copy, Clone)]
        pub enum $enum_name {
            $($(#[$variant_attr])* $variant = $value,)+
        }

        impl crate::prelude::DataConstant for $enum_name {

            type Primitive = $primitive_type;

            fn try_resolve(value: &Self::Primitive) -> Option<Self>
            {
                return match value {
                    $($value => Some(Self::$variant),)+
                    _ => None
                }
            }

            fn value(&self) -> $primitive_type {
                return *self as $primitive_type;
            }

            fn values() -> &'static [Self] {
                return [$(Self::$variant,)+].as_slice();
            }

        }
    };
}

pub(crate) use data_constants;

#[cfg(test)]
mod tests {
    use crate::prelude::DataConstant;

    data_constants!(Test<u16> {
        One = 1,
        Two = 2,
    });

    #[test]
    fn test_enum() {
        assert_eq!(Test::One.value(), 1);
        assert_eq!(Test::Two.value(), 2);

        assert_eq!(Test::try_resolve(&1), Some(Test::One));
        assert_eq!(Test::try_resolve(&2), Some(Test::Two));

        assert_eq!(Test::try_resolve(&0), None);
        assert_eq!(Test::try_resolve(&3), None);
    }
}
