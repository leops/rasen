use std::fmt;

#[derive(Eq, PartialEq, Hash)]
pub enum TypeName {
    Bool,
    Int(bool),
    Float(bool),
    Vec(u32, &'static TypeName),
    Mat(u32, &'static TypeName),
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
        match self {
            &TypeName::Bool => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_integer(&self) -> bool {
        match self {
            &TypeName::Int(_) => true,
            _ => false,
        }
    }
    #[inline]
    pub fn is_signed(&self) -> bool {
        match self {
            &TypeName::Int(true) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_float(&self) -> bool {
        match self {
            &TypeName::Float(_) => true,
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
}

impl fmt::Debug for TypeName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &TypeName::Bool => write!(f, "bool"),
            &TypeName::Int(true) => write!(f, "int"),
            &TypeName::Int(false) => write!(f, "uint"),
            &TypeName::Float(false) => write!(f, "float"),
            &TypeName::Float(true) => write!(f, "double"),

            &TypeName::Vec(len, scalar) => match scalar {
                &TypeName::Bool => write!(f, "bvec{}", len),
                &TypeName::Int(true) => write!(f, "ivec{}", len),
                &TypeName::Int(false) => write!(f, "uvec{}", len),
                &TypeName::Float(false) => write!(f, "vec{}", len),
                &TypeName::Float(true) => write!(f, "dvec{}", len),
                _ => Err(fmt::Error),
            },

            &TypeName::Mat(cols, scalar) => match scalar {
                &TypeName::Float(false) => write!(f, "mat{}", cols),
                &TypeName::Float(true) => write!(f, "dmat{}", cols),
                _ => Err(fmt::Error),
            },
        }
    }
}
