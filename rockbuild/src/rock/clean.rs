#![allow(non_snake_case)]
use std::io::Result;
use std::path::{PathBuf};
use crate::rock::misc;
use std::collections::HashMap;
use serde_json::Value;
use std::fs::*;

fn removeObjFile(src:&Vec<PathBuf>)
{
    for i in src {
        let mut obj:PathBuf = i.clone();
        obj.set_extension("o");
        match remove_file(obj)
        {
            Ok(_x) =>{}
            Err(_e) =>{}
        }
    }
}

fn cleanOption(optName: &PathBuf, fileMap: &HashMap<PathBuf, Value>) -> Result<String> {
    // println!("buildOption Entry optName:{:?}", optName);
    let fileJson: Value;

    match fileMap.get(optName) {
        Some(x) => {
            fileJson = x.clone();
            match x["DependRock"].as_array() {
                Some(z) => {
                    for i in z {
                        match i.as_str() {
                            Some(s) => {
                                cleanOption(&PathBuf::from(s), fileMap)?;
                            }
                            None => {}
                        }
                    }
                }
                None => {}
            }
        }
        None => {
            return Ok(String::from("Ok"));
        }
    }

    let fileAbsDir = PathBuf::from(optName.parent().unwrap());
    misc::cdDir(&fileAbsDir);
    misc::cmdSync(&misc::valueToVec(&fileJson["CmdStart"]), &fileAbsDir);
    let opt: misc::OptionData = misc::expandOption(&fileJson);

    let mut target:String;
    let mut targetExt:String;

    targetExt = opt.Target.clone();
    target = opt.Target.clone();
    if cfg!(target_os = "windows")
    {
        match PathBuf::from(&opt.Target).extension()
        {
            Some(x) =>{
                target = target.replace(&format!(".{:?}",x), "");
            }
            None =>{
                if opt.Type == "Program"
                {
                    targetExt.push_str(".exe");
                }
                else if opt.Type == "StaticLib"
                {
                    targetExt.push_str(".a");
                }
                else if opt.Type == "ShareLib"
                {
                    targetExt.push_str(".dll");
                }
            }
        }
    }
    else
    {
        match PathBuf::from(&opt.Target).extension()
        {
            Some(x) =>{
                target = target.replace(&format!(".{:?}",x), "");
            }
            None =>{
                if opt.Type == "Program"
                {
                    targetExt.push_str(".out");
                }
                else if opt.Type == "StaticLib"
                {
                    targetExt.push_str(".a");
                }
                else if opt.Type == "ShareLib"
                {
                    targetExt.push_str(".so");
                }
            }
        }
    }
    println!("Clean {:?}", optName);
    println!("{:?}", opt);
    removeObjFile(&opt.ASMSource);
    removeObjFile(&opt.CSource);
    removeObjFile(&opt.CXXSource);

    if opt.IsMap
    {
        match remove_file(format!("{}.map",target))
        {
            Ok(_) =>{}
            Err(_) =>{}
        }
    }

    if opt.IsAsm
    {
        match remove_file(format!("{}.asm",target))
        {
            Ok(_) =>{}
            Err(_) =>{}
        }
    }

    if opt.IsBinary
    {
        match remove_file(format!("{}.bin",target))
        {
            Ok(_) =>{}
            Err(_) =>{}
        }
    }

    match opt.Type.as_str() {
        "Program" => {
            match remove_file(targetExt)
            {
                Ok(_) =>{}
                Err(_) =>{}
            }
        }
        "StaticLib" => {
           match remove_file(targetExt)
           {
               Ok(_) =>{}
               Err(_) =>{}
           }
        }
        "ShareLib" => {
            match remove_file(targetExt)
            {
                Ok(_) =>{}
                Err(_) =>{}
            }
        }
        __ => {}
    }

    misc::cmdSync(&misc::valueToVec(&fileJson["CmdStop"]), &fileAbsDir);

    return Ok(String::from("Ok"));
}


pub fn clean(rockfile:PathBuf) -> Result<String>
{
    // println!("clean {}",rockfile);
    let fileAbsPath = misc::getAbsPath(&rockfile);
    let mut fileMap = HashMap::new();

    misc::getOption(&fileAbsPath, &mut fileMap);
    misc::initEnvs(
        &fileMap.get(&fileAbsPath).unwrap(),
        true,
        &misc::getAbsDir(&rockfile),
    );
    cleanOption(&fileAbsPath, &mut fileMap)?;

    return Ok(String::from("Ok"));
}
