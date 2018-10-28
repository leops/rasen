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
        env,
        ffi::OsStr,
        fmt,
        fs::OpenOptions,
        io::{prelude::*, ErrorKind, SeekFrom},
        path::PathBuf,
        process::Command,
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

    fn try_run_command(name: &str, args: &[&OsStr]) -> Result<bool, String> {
        let res = { Command::new(name).args(args).output() };

        match res {
            Ok(ref output) if output.status.success() => Ok(true),
            Ok(output) => Err(match str::from_utf8(&output.stderr) {
                Ok(stderr) => format!("Command {} failed:\n{}", name, stderr),
                Err(_) => format!("Command {} failed with output {:?}", name, output),
            }),

            Err(error) => match error.kind() {
                ErrorKind::NotFound => {
                    println!("Command {} not in path, skipping", name);
                    Ok(false)
                }
                _ => Err(format!("{:?}", error)),
            },
        }
    }

    fn file_compare_and_swap(path: &PathBuf, current: &ModuleWrapper, new: &str) {
        let mut file = {
            OpenOptions::new()
                .read(true)
                .write(true)
                .open(&path)
                .unwrap_or_else(|_| panic!("Could not open {}", path.display()))
        };

        let check_content = {
            let mut res = String::new();
            file.read_to_string(&mut res).unwrap();
            res
        };

        let current: String = current.to_string();
        if check_content != current && check_content != new {
            assert_eq!(
                check_content, current,
                "Test data have changed since compilation",
            );
            assert_eq!(
                check_content, new,
                "Test data has already been modified with another result",
            );
        }

        file.set_len(0).expect("truncate");
        file.seek(SeekFrom::Start(0)).expect("seek");
        write!(file, "{}", new).expect("write");
    }

    pub fn check_or_update_impl<T>(
        ref_name: &'static str,
        file: &'static str,
        ref_value: &'static ModuleWrapper,
        actual: T,
    ) where
        ModuleWrapper: From<T>,
    {
        let actual = ModuleWrapper::from(actual);
        if env::vars().any(|(name, val)| name == "RASEN_ALLOW_UPDATE" && val == "1") {
            if &actual != ref_value {
                println!("{} did not match test data, replacing ...", ref_name);

                let mut path = env::current_dir().expect("current_dir");
                path.pop();
                path.push(file);
                path.pop();
                path.push(ref_name);

                let assemble: String = actual.to_string();
                file_compare_and_swap(&path, ref_value, &assemble);

                let mut out_path = path.clone();
                out_path.set_file_name(
                    path.file_name()
                        .unwrap()
                        .to_string_lossy()
                        .replace("spvasm", "spv"),
                );

                let out_arg: &OsStr = "-o".as_ref();
                let out_path_arg = out_path.as_ref();
                let path_arg = path.as_ref();

                if try_run_command("spirv-as", &[out_arg, out_path_arg, path_arg]).unwrap() {
                    assert!(try_run_command("spirv-val", &[out_path_arg]).unwrap());
                    println!("Validated {}", ref_name);
                }
            }
        } else {
            assert_eq!(ref_value, &actual);
        }
    }
}

pub use self::_update_utils::build_module;

macro_rules! check_or_update {
    ($actual:expr, $reference:expr) => {{
        static REF: self::_update_utils::ModuleWrapper =
            { self::_update_utils::ModuleWrapper::Static(include_str!($reference)) };

        self::_update_utils::check_or_update_impl($reference, file!(), &REF, $actual)
    }};
}
