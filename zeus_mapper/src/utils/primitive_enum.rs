macro_rules! primitive_enum {
    ( $(#[derive($($derive:ident)+)])? $enum_name:tt { $($variant:ident = $value:expr,)+} ) => {
        #[derive(PartialEq, Debug, Copy, Clone)]
        pub enum $enum_name {
            $($variant = $value,)+
        }

        impl $enum_name {

            #![allow(warnings)]
            pub(crate) fn try_from<T>(value: T) -> Option<Self>
            where
                T: TryInto<isize>,
            {
                if let Ok(value) = value.try_into() {
                    match value {
                        $($value => return Some(Self::$variant),)+
                        _ => ()
                    }
                }
                return None;
            }

        }

    };
}

pub(crate) use primitive_enum;

#[cfg(test)]
mod tests {
    use std::io::Result;

    primitive_enum!(Test {
        One = 1,
        Two = 2,
    });

    #[test]
    fn test_enum() -> Result<()> {
        assert_eq!(Test::try_from(Test::One as isize).unwrap(), Test::One);
        assert_eq!(Test::try_from(Test::Two as isize).unwrap(), Test::Two);

        assert!(Test::try_from(0).is_none());
        assert_eq!(Test::try_from(1).unwrap(), Test::One);
        assert_eq!(Test::try_from(2).unwrap(), Test::Two);
        assert!(Test::try_from(3).is_none());

        return Ok(());
    }
}
