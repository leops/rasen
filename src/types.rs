use std::fmt;

/// Describes a SPIR-V data type
#[derive(Eq, PartialEq, Hash)]
pub enum TypeName {
    /// Basic boolean type
    Bool,
    /// Integer type, signed or not
    Int(bool /* is_signed */),
    /// Floating-point type, with single or double precision
    Float(bool /* is_double */),
    /// Vector type of n components of given scalar type
    Vec(u32 /* component_count */, &'static TypeName /* component_type */),
    /// Matrix type of n columns of given vector type
    Mat(u32 /* column_count */, &'static TypeName /* column_type */),
}

include!(concat!(env!("OUT_DIR"), "/types.rs"));

impl TypeName {
    pub const BOOL: &'static TypeName = &TypeName::Bool;
    pub const INT: &'static TypeName = &TypeName::Int(true);
    pub const UINT: &'static TypeName = &TypeName::Int(false);
    pub const FLOAT: &'static TypeName = &TypeName::Float(false);
    pub const DOUBLE: &'static TypeName = &TypeName::Float(true);

    #[inline]
    pub fn is_bool(&self) -> bool {
        match *self {
            TypeName::Bool => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_integer(&self) -> bool {
        match *self {
            TypeName::Int(_) => true,
            _ => false,
        }
    }
    #[inline]
    pub fn is_signed(&self) -> bool {
        match *self {
            TypeName::Int(true) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_float(&self) -> bool {
        match *self {
            TypeName::Float(_) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_num(&self) -> bool {
        self.is_integer() || self.is_float()
    }
    #[inline]
    pub fn is_scalar(&self) -> bool {
        self.is_bool() || self.is_num()
    }

    #[inline]
    pub fn size(&self) -> u32 {
        match *self {
            TypeName::Bool |
            TypeName::Int(_) |
            TypeName::Float(false) => 4,

            TypeName::Float(true) => 8,

            TypeName::Vec(len, ty) |
            TypeName::Mat(len, ty) => len * ty.size(),
        }
    }
}

impl fmt::Debug for TypeName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TypeName::Bool => write!(f, "bool"),
            TypeName::Int(true) => write!(f, "int"),
            TypeName::Int(false) => write!(f, "uint"),
            TypeName::Float(false) => write!(f, "float"),
            TypeName::Float(true) => write!(f, "double"),

            TypeName::Vec(len, scalar) => match *scalar {
                TypeName::Bool => write!(f, "bvec{}", len),
                TypeName::Int(true) => write!(f, "ivec{}", len),
                TypeName::Int(false) => write!(f, "uvec{}", len),
                TypeName::Float(false) => write!(f, "vec{}", len),
                TypeName::Float(true) => write!(f, "dvec{}", len),
                _ => Err(fmt::Error),
            },

            TypeName::Mat(columns, vec) => match *vec {
                TypeName::Vec(rows, &TypeName::Float(false)) if columns == rows => write!(f, "mat{}", rows),
                TypeName::Vec(rows, &TypeName::Float(true)) if columns == rows => write!(f, "dmat{}", rows),
                _ => Err(fmt::Error),
            },
        }
    }
}
