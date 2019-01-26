mod _update_utils {
    use rasen::{
        errors::{self, Error},
        prelude::ModuleBuilder,
    };
    use rspirv::{
        binary::{Assemble, Disassemble},
        mr::{load_words, Module as SpirvModule},
    };
    use std::{
        convert::TryFrom,
        fmt,
        str,
    };

    pub enum ModuleWrapper {
        Module(Box<SpirvModule>),
        String(String),
        Static(&'static str),
    }

    impl Clone for ModuleWrapper {
        fn clone(&self) -> ModuleWrapper {
            match *self {
                ModuleWrapper::Module(ref module) => {
                    ModuleWrapper::Module(Box::new(load_words(module.assemble()).unwrap()))
                }
                ModuleWrapper::String(ref string) => ModuleWrapper::String(string.clone()),
                ModuleWrapper::Static(string) => ModuleWrapper::Static(string),
            }
        }
    }

    impl<'a> From<&'a ModuleWrapper> for ModuleWrapper {
        fn from(other: &'a ModuleWrapper) -> ModuleWrapper {
            other.clone()
        }
    }

    impl From<String> for ModuleWrapper {
        fn from(string: String) -> ModuleWrapper {
            ModuleWrapper::String(string)
        }
    }

    impl ToString for ModuleWrapper {
        fn to_string(&self) -> String {
            match *self {
                ModuleWrapper::Module(ref module) => module.disassemble(),
                ModuleWrapper::String(ref string) => string.clone(),
                ModuleWrapper::Static(string) => string.into(),
            }
        }
    }

    impl fmt::Debug for ModuleWrapper {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            write!(fmt, "{}", self.to_string())
        }
    }

    impl PartialEq for ModuleWrapper {
        fn eq(&self, other: &Self) -> bool {
            let this: String = self.to_string();
            let other: String = other.to_string();
            this == other
        }
    }

    pub fn build_module<'a, I, T>(graph: &'a I, mod_type: T) -> errors::Result<ModuleWrapper>
    where
        ModuleBuilder: TryFrom<(&'a I, T), Error = Error>,
    {
        Ok(ModuleWrapper::Module(Box::new(
            ModuleBuilder::try_from((graph, mod_type))?.build()?,
        )))
    }
}

pub use self::_update_utils::build_module;

macro_rules! assert_spirv_snapshot_matches {
    ($reference:expr, $actual:expr) => {{
        use insta::assert_snapshot_matches;
        let value = self::_update_utils::ModuleWrapper::from($actual).to_string();
        assert_snapshot_matches!($reference, value, stringify!($actual))
    }};
}
