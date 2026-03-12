/// Versioned sum type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum V1_2<V1, V2> {
    V1(V1),
    V2(V2),
}

#[macro_export]
macro_rules! v1_2_each_field {
    ($ag:expr, $field:ident) => {
        match $ag {
            $crate::typedefs::versioned::V1_2::V1(p) => &p.$field,
            $crate::typedefs::versioned::V1_2::V2(p) => &p.$field,
        }
    };
}

#[macro_export]
macro_rules! v1_2_each_meth {
    ($ag:expr, $meth:ident) => {
        match $ag {
            $crate::typedefs::versioned::V1_2::V1(p) => p.$meth(),
            $crate::typedefs::versioned::V1_2::V2(p) => p.$meth(),
        }
    };
}

#[macro_export]
macro_rules! v1_2_each_field_mut {
    ($ag:expr, $field:ident) => {
        match $ag {
            $crate::typedefs::versioned::V1_2::V1(p) => &mut p.$field,
            $crate::typedefs::versioned::V1_2::V2(p) => &mut p.$field,
        }
    };
}
