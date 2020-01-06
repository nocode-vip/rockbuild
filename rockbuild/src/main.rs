use std::env;
use std::path::PathBuf;
// use std::collections::HashMap;
// use std::result::Result;
// use std::option;
// use std::process::Command;

mod rock;
use crate::rock::{build,clean};

fn help()
{
    let d = "Usage: rock options [rockfile]
    Options:
            build   Building c/c++
            rebuild Clean object and building c/c++
            clean   Clean object
            version Display rockbuild version information";
    println!("{}", d);
}

fn main() {
    let mut arguments = vec![];

    for argument in env::args() {
        arguments.push(argument);
    }

    if arguments.len()<=1
    {
        return help()
    }

    // println!("main current dir {:?}", env::current_dir().unwrap().canonicalize().unwrap());

    // let cfile = PathBuf::from(&arguments[2]);
    // println!("{:?}-{:?}", cfile,cfile.parent().unwrap().canonicalize().unwrap());

    match arguments[1].as_str() 
    {
        "build" => {
            match build::building(PathBuf::from(&arguments[2]),false)
            {
                Ok(x) => {
                    println!("Building Success {}!!!\n",x);
                    return;
                },
                Err(e) =>{
                    println!("Building Error {:?}\n",e);
                    return;
                }
            }
        },
        "rebuild" => {
            match build::building(PathBuf::from(&arguments[2]),true)
            {
                Ok(x) => {
                    println!("Rebuilding Success {}!!!\n",x);
                    return;
                },
                Err(e) =>{
                    println!("Rebuilding Error {:?}\n",e);
                    return;
                }
            }
        },
        "clean" => {
            match clean::clean(PathBuf::from(&arguments[2]))
            {
                Ok(x) => {
                    println!("Clean Success {}!!\n",x);
                    return;
                },
                Err(e) => {
                    println!("Clean Error {:?}\n",e);
                    return;
                }
            }
        },
        _ => help(),
    }
}
