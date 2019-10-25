pub use spirv_headers::Dim;
use spirv_headers::StorageClass;
use std::fmt;

/// Describes a SPIR-V data type
#[derive(Eq, PartialEq, Hash)]
pub enum TypeName {
    Void,
    /// Basic boolean type
    Bool,
    /// Integer type, signed or not
    Int(bool /* is_signed */),
    /// Floating-point type, with single or double precision
    Float(bool /* is_double */),
    /// Vector type of n components of given scalar type
    Vec(
        u32,               /* component_count */
        &'static TypeName, /* component_type */
    ),
    /// Matrix type of n columns of given vector type
    Mat(
        u32,               /* column_count */
        &'static TypeName, /* column_type */
    ),
    /// Composite type of an image and an actual sampler object
    Sampler(
        &'static TypeName, /* sampled_type */
        Dim,               /* dimensionality */
    ),

    #[doc(hidden)]
    _Pointer(&'static TypeName, StorageClass),
}

include!(concat!(env!("OUT_DIR"), "/types.rs"));

impl TypeName {
    pub const VOID: &'static Self = &TypeName::Void;
    pub const BOOL: &'static Self = &TypeName::Bool;
    pub const INT: &'static Self = &TypeName::Int(true);
    pub const UINT: &'static Self = &TypeName::Int(false);
    pub const FLOAT: &'static Self = &TypeName::Float(false);
    pub const DOUBLE: &'static Self = &TypeName::Float(true);

    pub(crate) const FLOAT_PTR_UNI: &'static Self =
        &TypeName::_Pointer(Self::FLOAT, StorageClass::Uniform);
    pub(crate) const FLOAT_PTR_FUN: &'static Self =
        &TypeName::_Pointer(Self::FLOAT, StorageClass::Function);

    #[inline]
    pub(crate) fn is_integer(&self) -> bool {
        match *self {
            TypeName::Int(_) => true,
            _ => false,
        }
    }
    #[inline]
    pub(crate) fn is_signed(&self) -> bool {
        match *self {
            TypeName::Int(true) => true,
            _ => false,
        }
    }

    #[inline]
    pub(crate) fn is_float(&self) -> bool {
        match *self {
            TypeName::Float(_) => true,
            _ => false,
        }
    }

    #[inline]
    pub(crate) fn is_num(&self) -> bool {
        self.is_integer() || self.is_float()
    }

    #[inline]
    pub(crate) fn size(&self) -> u32 {
        match *self {
            TypeName::Void | TypeName::Sampler(..) | TypeName::_Pointer(..) => 0,

            TypeName::Bool | TypeName::Int(_) | TypeName::Float(false) => 4,

            TypeName::Float(true) => 8,

            TypeName::Vec(len, ty) | TypeName::Mat(len, ty) => len * ty.size(),
        }
    }
}

fn print_type_prefix(f: &mut fmt::Formatter, ty: &TypeName) -> fmt::Result {
    match *ty {
        TypeName::Bool => write!(f, "b"),
        TypeName::Int(true) => write!(f, "i"),
        TypeName::Int(false) => write!(f, "u"),
        TypeName::Float(false) => Ok(()),
        TypeName::Float(true) => write!(f, "d"),
        _ => Err(fmt::Error),
    }
}

impl fmt::Debug for TypeName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TypeName::Void => write!(f, "void"),
            TypeName::Bool => write!(f, "bool"),
            TypeName::Int(true) => write!(f, "int"),
            TypeName::Int(false) => write!(f, "uint"),
            TypeName::Float(false) => write!(f, "float"),
            TypeName::Float(true) => write!(f, "double"),

            TypeName::Vec(len, scalar) => {
                print_type_prefix(f, scalar)?;
                write!(f, "vec{}", len)
            }

            TypeName::Mat(columns, vec) => match *vec {
                TypeName::Vec(rows, &TypeName::Float(false)) if columns == rows => {
                    write!(f, "mat{}", rows)
                }
                TypeName::Vec(rows, &TypeName::Float(true)) if columns == rows => {
                    write!(f, "dmat{}", rows)
                }
                _ => Err(fmt::Error),
            },

            TypeName::Sampler(sampled_type, dimensionality) => {
                print_type_prefix(f, sampled_type)?;
                write!(f, "sampler")?;
                match dimensionality {
                    Dim::Dim1D => write!(f, "1D"),
                    Dim::Dim2D => write!(f, "2D"),
                    Dim::Dim3D => write!(f, "3D"),
                    Dim::DimCube => write!(f, "Cube"),
                    Dim::DimRect => write!(f, "2DRect"),
                    Dim::DimBuffer => write!(f, "Buffer"),
                    Dim::DimSubpassData => write!(f, "SubpassData"),
                }
            }

            TypeName::_Pointer(inner, _) => write!(f, "&{:?}", inner),
        }
    }
}
