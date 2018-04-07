use std::fmt;
use spirv_headers::Dim;

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
    Vec(u32 /* component_count */, &'static TypeName /* component_type */),
    /// Matrix type of n columns of given vector type
    Mat(u32 /* column_count */, &'static TypeName /* column_type */),
    /// Composite type of an image and an actual sampler object
    Sampler(&'static TypeName /* sampled_type */, Dim /* dimensionality */),
}

include!(concat!(env!("OUT_DIR"), "/types.rs"));

impl TypeName {
    pub const VOID: &'static TypeName = &TypeName::Void;
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
            TypeName::Void |
            TypeName::Sampler(_, _) => 0,

            TypeName::Bool |
            TypeName::Int(_) |
            TypeName::Float(false) => 4,

            TypeName::Float(true) => 8,

            TypeName::Vec(len, ty) |
            TypeName::Mat(len, ty) => len * ty.size(),
        }
    }
}

fn print_type_prefix(f: &mut fmt::Formatter, ty: &TypeName) -> fmt::Result {
    match *ty {
        TypeName::Bool => write!(f, "b"),
        TypeName::Int(true) => write!(f, "i"),
        TypeName::Int(false) => write!(f, "u"),
        TypeName::Float(false) => write!(f, "v"),
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
            },

            TypeName::Mat(columns, vec) => match *vec {
                TypeName::Vec(rows, &TypeName::Float(false)) if columns == rows => write!(f, "mat{}", rows),
                TypeName::Vec(rows, &TypeName::Float(true)) if columns == rows => write!(f, "dmat{}", rows),
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
            },
        }
    }
}
