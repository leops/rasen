extern crate diff;

use std::env;
use std::path::PathBuf;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::{SeekFrom, ErrorKind};
use std::process::Command;
use std::ffi::OsStr;
use std::fmt::Write as FmtWrite;
use diff::Result as Res;

fn try_run_command(name: &str, args: &[&OsStr]) -> Result<bool, String> {
    let res = {
        Command::new(name)
            .args(args)
            .status()
    };

    match res {
        Ok(status) if status.success() => Ok(true),
        Ok(status) => Err(format!("Command {} failed with error {:?}", name, status)),

        Err(error) => match error.kind() {
            ErrorKind::NotFound => {
                println!("Command {} not in path, skipping", name);
                Ok(false)
            },
            _ => Err(format!("{:?}", error)),
        },
    }
}

fn file_compare_and_swap(path: &PathBuf, current: &str, new: &str) {
    let mut file = {
        OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .expect(&format!("Could not open {}", path.display()))
    };

    let check_content = {
        let mut res = String::new();
        file.read_to_string(&mut res).unwrap();
        res
    };

    assert!(check_content == current, "Test data have changed since compilation");

    file.set_len(0).expect("truncate");
    file.seek(SeekFrom::Start(0)).expect("seek");
    write!(file, "{}", new).expect("write");
}

macro_rules! check_or_update {
    ($actual:ident, $reference:expr) => {{
        static REF: &'static str = include_str!($reference);

        if env::vars().any(|(name, val)| name == "RASEN_ALLOW_UPDATE" && val == "1") {
            if $actual != REF {
                println!("{} did not match test data, replacing ...", $reference);

                let mut path = env::current_dir().expect("current_dir");
                path.pop();
                path.push(file!());
                path.pop();
                path.push($reference);

                file_compare_and_swap(&path, REF, &$actual);

                let mut out_path = path.clone();
                out_path.set_file_name(
                    path.file_name().unwrap()
                        .to_string_lossy()
                        .replace("spvasm", "spv")
                );

                let out_arg: &OsStr = "-o".as_ref();
                let out_path_arg = out_path.as_ref();
                let path_arg = path.as_ref();

                if try_run_command("spirv-as", &[out_arg, out_path_arg, path_arg]).unwrap() {
                    assert!(try_run_command("spirv-val", &[out_path_arg]).unwrap());
                    println!("Validated {}", $reference);
                }
            }
        } else {
            if $actual != REF {
                let max_len = {
                    REF.split("\n")
                        .map(|line| line.len())
                        .max()
                        .unwrap()
                };

                let mut res = String::new();
                for diff in diff::lines(REF, &$actual) {
                    match diff {
                        Res::Both(a, b) => {
                            writeln!(
                                res, "{}{}{}",
                                a, " ".repeat(max_len - a.len()), b,
                            ).unwrap();
                        },
                        Res::Right(b) => {
                            writeln!(
                                res, "{}{}",
                                " ".repeat(max_len),
                                b,
                            ).unwrap();
                        },
                        Res::Left(a) => {
                            writeln!(
                                res, "{}", a,
                            ).unwrap();
                        },
                    }
                }

                panic!("actual != ref\n{}", res)
            }
        }
    }};
}
