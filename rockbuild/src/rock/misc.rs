#![allow(non_snake_case)]
use serde_json::{json, Value};
use std::env;
use glob::glob;
use lazy_static::*;
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader,Result,Error,ErrorKind};
use std::path::PathBuf;
// use std::path::Prefix::*;
// use std::path::{Component, Path, Prefix};
use std::process::{Command,Child};
use num_cpus;

#[derive(Debug,Clone)]
pub struct OptionData {
    pub Target: String,
    pub Version: String,
    pub Type: String,
    pub DependRock: Vec<PathBuf>,
    pub ASMSource: Vec<PathBuf>,
    pub CSource: Vec<PathBuf>,
    pub CXXSource: Vec<PathBuf>,
    pub DependObject: Vec<PathBuf>,
    pub Object: Vec<PathBuf>,
    
    pub CmdStart: Vec<String>,
    pub CmdStop: Vec<String>,
    
    pub Rebuild: bool,
    pub IsMap: bool,
    pub IsAsm: bool,
    pub IsBinary: bool,
    pub IsStrip: bool,
    pub IsSilent: bool,
    pub Jobs: u64,
    
    pub CROSSCOMPILE: String,
    pub AS: String,
    pub CC: String,
    pub CXX: String,
    pub LD: String,
    pub AR: String,
    pub STRIP: String,
    pub OBJCOPY: String,
    pub OBJDUMP: String,
    pub AFLAGS: String,
    pub CFLAGS: String,
    pub CXXFLAGS: String,
    pub LDFLAGS: String,
    pub DEFS: String,
    pub LIBS: String,
    pub LIBPATH: String,
    pub INCLUDES: String,
}

pub fn getAbsPath(path: &PathBuf) -> PathBuf {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^\\\\\?\\").unwrap();
    }

    if cfg!(target_os = "windows") {
        let tpath;
        if path.is_relative() {
            match path.canonicalize()
            {
                Ok(x) => tpath = x.clone(),
                Err(_e) => return path.clone(),
            }
        } else {
            tpath = path.clone();
        }

        let res = RE.replace(tpath.to_str().unwrap(), "");
        return PathBuf::from(String::from(res));
    } else {
        let tpath;
        if path.is_relative() {
            match path.canonicalize()
            {
                Ok(x) => tpath = x.clone(),
                Err(_e) => return path.clone(),
            }
        } else {
            tpath = path.clone();
        }

        return tpath;
    }
}

pub fn getAbsDir(path: &PathBuf) -> PathBuf {
    match getAbsPath(path).parent()
    {
        Some(x) => return x.to_path_buf().clone(),
        None => panic!("getAbsDir Error"),
    }
}

fn expandRockGlob(cfg: &str, out: &mut Vec<PathBuf>) {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^@\{RockGlob\(([^\}]+)\)\}$").unwrap();
    }

    match RE.captures(cfg) {
        Some(x) => {
            // println!("glob: len:{},0:{:?},1:{:?}",x.len(), x.get(0).unwrap().as_str(),x.get(1).unwrap().as_str());
            for entry in glob(x.get(1).unwrap().as_str()).expect("Failed to read glob pattern") {
                match entry {
                    Ok(path) => {
                        out.push(getAbsPath(&path));
                    }
                    Err(e) => {
                        println!("{:?}", e);
                    }
                }
            }
        }
        None => {
            out.push(getAbsPath(&PathBuf::from(cfg)));
        }
    }
}

fn expandVarRegex(cfg:&str)->Option<String>
{
    lazy_static! {
        static ref RE: Regex = Regex::new(r"#\{([^\}]+)\}").unwrap();
    }

    let mut v:String = String::from(cfg);

    for caps in RE.captures_iter(cfg) {
        // println!("s0:{:?}",caps.len());
        // println!("s0:{:?}",caps.get(0).unwrap().as_str());
        // println!("s0:{:?}",caps.get(1).unwrap().as_str());

        match env::var(caps.get(1).unwrap().as_str())
        {
            Ok(x) =>{
                v = v.replace(caps.get(0).unwrap().as_str(),&x);
                return Some(v);
            }
            Err(_e) =>{
                return None;
            }
        }
    }

    return None;
}

pub fn valueToVec(v:&Value)->Vec<String>
{
    let mut ret = vec![];

    if v.is_string()
    {
        match v.as_str()
        {
            Some(x) =>{
                ret.push(String::from(x));
            }
            None =>{}
        }
    }

    match v.as_array()
    {
        Some(x) => {
            for i in x {
                match i.as_str()
                {
                    Some(s) => {
                        ret.push(String::from(s));
                    }
                    None =>{}
                }
            }
        }
        None =>{}
    }

    return ret;
}

pub fn cdDir(dir: &PathBuf) {
    // println!("{:?}", dir);
    // println!("{:?}", env::current_dir().unwrap());
    // println!("{:?}", env::current_exe().unwrap());

    match env::set_current_dir(dir.as_path()) {
        Err(e) => {
            panic!("set current dir error!--{}", e);
        }
        Ok(_x) => {
            // println!("set curent dir {:?}", env::current_dir().unwrap());
        }
    }
}

pub fn getOption(fileAbsPath: &PathBuf, fileMap: &mut HashMap<PathBuf, Value>) {
    let fileAbsDir;

    if fileAbsPath.is_relative() {
        // fileAbsPath = file.canonicalize().unwrap();
        // fileAbsDir = file.parent().unwrap().canonicalize().unwrap();
        panic!("{:?} is not abs path", fileAbsPath);
    }

    // fileAbsPath = file.clone();
    fileAbsDir = getAbsDir(fileAbsPath);
    let curDir = env::current_dir().unwrap();
    let mut fileJson: Value;
    let mut rock = vec![];
    cdDir(&fileAbsDir);

    match File::open(&fileAbsPath) {
        Ok(x) => match serde_json::from_reader(BufReader::new(x)) {
            Ok(j) => {
                fileJson = j;
            }
            Err(e) => {
                panic!("Read Json from {:?} Error:{}", fileAbsPath, e);
            }
        },
        Err(e) => {
            panic!("Read File {:?} Error:{:?}", fileAbsPath, e);
        }
    }

    // println!("{:?}", fileJson);

    match fileJson["DependRock"].as_array() {
        Some(x) => {
            for i in x {
                match i.as_str() {
                    Some(s) => {
                        expandRockGlob(&s, &mut rock);
                    }
                    None => {}
                }
            }
            *fileJson.get_mut("DependRock").unwrap() = json!(rock);
        }
        None => {}
    }

    // println!("{:?}", fileJson);
    // println!("{:?}", fileJson["DependRock"][0].as_str().unwrap());

    fileMap.insert(fileAbsPath.clone(), fileJson.clone());

    // println!("fileMap:{:?}", fileMap);

    match fileJson["DependRock"].as_array() {
        Some(x) => {
            for i in x {
                match i.as_str() {
                    Some(s) => {
                        getOption(&PathBuf::from(s), fileMap);
                    }
                    None => {}
                }
            }
        }
        None => {}
    }
    cdDir(&curDir);

    // println!("{:?}", fileAbsPath);
    // println!("{:?}", fileJson);
}

fn expandVarWithPrefix(defs:&Value,prefix:&str,isGlob:bool)->Option<String>
{
    let mut ret:String=String::new();

    for i in valueToVec(defs) {
        match expandVarRegex(i.as_str())
        {
            Some(x) =>{
                if isGlob == false
                {
                    if ret.len()>0
                    {
                        ret.push_str(" ");
                    }
                    // ret.push_str(prefix);
                    ret.push_str(x.as_str());
                }
                else if x.len()>0
                {
                    let mut tmp:Vec<PathBuf> = vec![];
                    expandRockGlob(x.as_str(), &mut tmp);
                    for y in tmp {
                        if ret.len()>0
                        {
                            ret.push_str(" ");
                        }
                        // ret.push_str(prefix);
                        ret.push_str(y.to_str().unwrap());
                    }
                }
            }
            None =>{
                if isGlob == false
                {
                    if ret.len()>0
                    {
                        ret.push_str(" ");
                    }
                    ret.push_str(prefix);
                    ret.push_str(i.as_str());
                }
                else if i.len()>0
                {
                    let mut tmp:Vec<PathBuf> = vec![];
                    expandRockGlob(i.as_str(), &mut tmp);
                    for y in tmp {
                        if ret.len()>0
                        {
                            ret.push_str(" ");
                        }
                        ret.push_str(prefix);
                        ret.push_str(y.to_str().unwrap());
                    }
                }
            }
        }
    }

    if ret.len() == 0
    {
        return None;
    }
    else
    {
        return Some(ret);
    }
}

pub fn initEnvs(rootJson: &Value,rebuild:bool,dir:&PathBuf) {
    let setvar_str = |c: &Value, k: &str,d:&str| match c[k].as_str() {
        Some(x) => {
            env::set_var(k, x);
        }
        None => {
            match env::var(k)
            {
                Ok(_y) =>{}
                Err(_e) => env::set_var(k,d),
            }
        }
    };

    let setvar_bool = |c: &Value, k: &str,d:bool| match c[k].as_bool() {
        Some(x) => {
            env::set_var(k, format!("{}",x));
        }
        None => {
            match env::var(k)
            {
                Ok(_y) =>{}
                Err(_e) => env::set_var(k,format!("{}",d)),
            }
        }
    };

    let setvar_u64 = |c: &Value, k: &str,d:u64| match c[k].as_u64() {
        Some(x) => {
            env::set_var(k, format!("{}",x));
        }
        None => {
            match env::var(k)
            {
                Ok(_y) =>{}
                Err(_e) => env::set_var(k,format!("{}",d)),
            }
        }
    };

    let setvar_option = |o:&Option<String>,k:&str,d:&str| match o {
        Some(x) => {
            env::set_var(k, x);
        }
        None => {
            match env::var(k)
            {
                Ok(_y) =>{}
                Err(_e) => env::set_var(k,d),
            }
        }
    };

    // println!("cpu nums:{:?}",num_cpus::get());

    let curDir = env::current_dir().unwrap();
    cdDir(dir);

    cmdSync(&valueToVec(&rootJson["ENVS"]["CmdStart"]), dir);
 
    setvar_bool(&rootJson["ENVS"],"IsStrip",true);
    setvar_u64(&rootJson["ENVS"],"Jobs",num_cpus::get() as u64);
    setvar_bool(&rootJson["ENVS"],"IsSilent",false);
    env::set_var("Rebuild", format!("{}",rebuild));

    setvar_str(&rootJson["ENVS"], "CROSSCOMPILE","");
    setvar_str(&rootJson["ENVS"], "AS","gcc");
    setvar_str(&rootJson["ENVS"], "CC","gcc");
    setvar_str(&rootJson["ENVS"], "CXX","gcc");
    setvar_str(&rootJson["ENVS"], "LD","gcc");
    setvar_str(&rootJson["ENVS"], "AR","ar");
    setvar_str(&rootJson["ENVS"], "STRIP","strip");
    setvar_str(&rootJson["ENVS"], "OBJCOPY","objcopy");
    setvar_str(&rootJson["ENVS"], "OBJDUMP","objdump");
    setvar_option(&expandVarWithPrefix(&rootJson["ENVS"]["AFLAGS"],"",false),"AFLAGS","");
    setvar_option(&expandVarWithPrefix(&rootJson["ENVS"]["CFLAGS"],"",false),"CFLAGS","");
    setvar_option(&expandVarWithPrefix(&rootJson["ENVS"]["CXXFLAGS"],"",false),"CXXFLAGS","");
    setvar_option(&expandVarWithPrefix(&rootJson["ENVS"]["LDFLAGS"],"",false),"LDFLAGS","");
    // setvar_str(&rootJson["ENVS"], "AFLAGS","");
    // setvar_str(&rootJson["ENVS"], "CFLAGS","");
    // setvar_str(&rootJson["ENVS"], "CXXFLAGS","");
    // setvar_str(&rootJson["ENVS"], "LDFLAGS","");
    setvar_option(&expandVarWithPrefix(&rootJson["ENVS"]["DEFS"],"-D",false),"DEFS","");
    setvar_option(&expandVarWithPrefix(&rootJson["ENVS"]["LIBS"], "-l",true),"LIBS","");
    setvar_option(&expandVarWithPrefix(&rootJson["ENVS"]["LIBPATH"], "-L",true),"LIBPATH","");
    setvar_option(&expandVarWithPrefix(&rootJson["ENVS"]["INCLUDES"], "-I",true),"INCLUDES","");

    // for (key, value) in env::vars() 
    // {
    //     println!("{}: {}", key, value);
    // }

    cmdSync(&valueToVec(&rootJson["ENVS"]["CmdStop"]), dir);
    cdDir(&curDir);
}


pub fn expandOption(fileJson: &Value) -> OptionData {
    let mut data = OptionData {
        Target: String::from("unknow"),
        Version: String::from("1.0.0"),
        Type: String::from("Program"),
        DependRock: vec![],
        ASMSource: vec![],
        CSource: vec![],
        CXXSource: vec![],
        DependObject: vec![],
        Object: vec![],
        
        CmdStart: vec![],
        CmdStop: vec![],

        Rebuild: false,
        IsMap: false,
        IsAsm: false,
        IsBinary : false,
        IsSilent: false,
        IsStrip: true,
        Jobs: 4,
        
        CROSSCOMPILE: String::new(),
        AS: String::from("gcc"),
        AR: String::from("ar"),
        CC: String::from("gcc"),
        CXX: String::from("gcc"),
        LD: String::from("gcc"),
        STRIP: String::from("strip"),
        OBJCOPY: String::from("objcopy"),
        OBJDUMP: String::from("objdump"),
        AFLAGS: String::from(""),
        CFLAGS: String::from(""),
        CXXFLAGS: String::from(""),
        LDFLAGS: String::new(),
        DEFS: String::new(),
        LIBS: String::new(),
        LIBPATH: String::new(),
        INCLUDES: String::new(),
    };

    let getvar_bool = |v:&Value,k:&str,d:bool|->bool
    {
        match v[k].as_bool()
        {
            Some(x) =>{return x;}
            None =>{
                match env::var(k)
                {
                    Ok(y) =>{
                        return y.parse().unwrap_or(d);
                    }
                    Err(_e) =>{
                        return d;
                    }
                }
            }
        }
    };

    let getvar_u64 = |v:&Value,k:&str,d:u64|->u64
    {
        match v[k].as_u64()
        {
            Some(x) =>{return x;}
            None =>{
                match env::var(k)
                {
                    Ok(y) =>{
                        return y.parse().unwrap_or(d);
                    }
                    Err(_e) =>{
                        return d;
                    }
                }
            }
        }
    };

    let getvar_str = |v:&Value,k:&str,d:&str|->String
    {
        match v[k].as_str()
        {
            Some(x) =>{return String::from(x);}
            None =>{
                match env::var(k)
                {
                    Ok(y) =>{
                        return y.clone();
                    }
                    Err(_e) =>{
                        return String::from(d);
                    }
                }
            }
        }
    };

    
    match fileJson["Type"].as_str() {
        Some(x) => {
            data.Type = String::from(x);
        }
        None => {
            panic!("ERROR:No \"Type\" Config!!!");
        }
    }
   
    match fileJson["Target"].as_str() {
        Some(x) => {
            data.Target = String::from(x);
        }
        None => {}
    }

    match fileJson["Version"].as_str() {
        Some(x) => {
            data.Version = String::from(x);
        }
        None => {}
    }

    for s in valueToVec(&fileJson["DependObject"]) {
        expandRockGlob(&s, &mut data.DependObject);
    }
    
    for s in valueToVec(&fileJson["ASMSource"]) {
        expandRockGlob(&s, &mut data.ASMSource);
    }
    
    for s in valueToVec(&fileJson["CSource"]) {
        expandRockGlob(&s, &mut data.CSource);
    }

    for s in valueToVec(&fileJson["CXXSource"]) {
        expandRockGlob(&s, &mut data.CXXSource);
    }

    // data.CmdStart.append(&mut valueToVec(&fileJson["CmdStart"]));
    data.CmdStop.append(&mut valueToVec(&fileJson["CmdStop"]));
    
    data.Rebuild = getvar_bool(&fileJson,"Rebuild",false);
    data.IsMap = getvar_bool(&fileJson,"IsMap",false);
    data.IsAsm = getvar_bool(&fileJson,"IsAsm",false);
    data.IsBinary = getvar_bool(&fileJson,"IsBinary",false);
    data.IsStrip = getvar_bool(&fileJson,"IsStrip",true);
    data.IsSilent = getvar_bool(&fileJson,"IsSilent",false);
    data.Jobs = getvar_u64(&fileJson,"Jobs",4);

    data.CROSSCOMPILE = getvar_str(&fileJson,"CROSSCOMPILE","");
    data.AS = getvar_str(&fileJson,"AS","gcc");
    data.AR = getvar_str(&fileJson,"AR","ar");
    data.CC = getvar_str(&fileJson,"CC","gcc");
    data.CXX = getvar_str(&fileJson,"CXX","gcc");
    data.LD = getvar_str(&fileJson,"LD","gcc");
    data.STRIP = getvar_str(&fileJson,"STRIP","strip");
    data.OBJCOPY = getvar_str(&fileJson,"OBJCOPY","objcopy");
    data.OBJDUMP = getvar_str(&fileJson,"OBJDUMP","objdump");
    data.AFLAGS = getvar_str(&fileJson,"AFLAGS","");
    data.CFLAGS = getvar_str(&fileJson,"CFLAGS","");
    data.CXXFLAGS = getvar_str(&fileJson,"CXXFLAGS","");

    match expandVarWithPrefix(&fileJson["AFLAGS"], "",false)
    {
        Some(x) => data.AFLAGS = x.clone(),
        None =>{
            data.AFLAGS = getvar_str(&fileJson,"AFLAGS","");
        },
    }

    match expandVarWithPrefix(&fileJson["CFLAGS"], "",false)
    {
        Some(x) => data.CFLAGS = x.clone(),
        None =>{
            data.CFLAGS = getvar_str(&fileJson,"CFLAGS","");
        },
    }


    match expandVarWithPrefix(&fileJson["CXXFLAGS"], "",false)
    {
        Some(x) => data.CXXFLAGS = x.clone(),
        None =>{
            data.CXXFLAGS = getvar_str(&fileJson,"CXXFLAGS","");
        },
    }

    match expandVarWithPrefix(&fileJson["LDFLAGS"], "",false)
    {
        Some(x) => data.LDFLAGS = x.clone(),
        None =>{
            data.LDFLAGS = getvar_str(&fileJson,"LDFLAGS","");
        },
    }

    match expandVarWithPrefix(&fileJson["DEFS"], "-D",false)
    {
        Some(x) => data.DEFS = x.clone(),
        None =>{
            data.DEFS = getvar_str(&fileJson,"DEFS","");
            if !data.DEFS.is_empty()
            {
                data.DEFS.push_str(format!(" -DTarget={}",data.Target).as_str());
                data.DEFS.push_str(format!(" -DVersion={}",data.Version).as_str());
            }
            else
            {
                data.DEFS.push_str(format!("-DTarget={}",data.Target).as_str());
                data.DEFS.push_str(format!(" -DVersion={}",data.Version).as_str());
            }
        },
    }

    match expandVarWithPrefix(&fileJson["LIBS"], "-l",true)
    {
        Some(x) => data.LIBS = x.clone(),
        None =>{
            data.LIBS = getvar_str(&fileJson,"LIBS","");
        },
    }

    match expandVarWithPrefix(&fileJson["LIBPATH"], "-L",true)
    {
        Some(x) => data.LIBPATH = x.clone(),
        None =>{
            data.LIBPATH = getvar_str(&fileJson,"LIBPATH","");
        },
    }

    match expandVarWithPrefix(&fileJson["INCLUDES"], "-I",true)
    {
        Some(x) => data.INCLUDES = x.clone(),
        None =>{
            data.INCLUDES = getvar_str(&fileJson,"INCLUDES","");
        },
    }

    return data;
}

pub fn execAsync(cmd: &str, cmdArgs:String,silent:bool) ->Result<Child>{
    
    if silent==false
    {
        println!("{:?} {:?}",cmd,cmdArgs);
    }
    let arg:Vec<&str> = cmdArgs.split(' ').collect();

    return Command::new(cmd)
                .args(arg)
                .spawn();
                // .expect("failed to execute process");
}

pub fn execSync(cmd: &str, cmdArgs:String,silent:bool)->Result<String>
{
    if silent==false
    {
        println!("{:?} {:?}",cmd,cmdArgs);
    }
    let arg:Vec<&str> = cmdArgs.split(' ').collect();

    let status = Command::new(cmd)
                .args(arg)
                .status()
                .expect("failed to execute process");
    if status.success()
    {
        return Ok(String::from("Ok"));
    }
    else
    {
        return Err(Error::from(ErrorKind::Other));
    }
}


pub fn cmdSync(cmdArray:&Vec<String>,dir:&PathBuf)
{
    let runCmd = |cmd: &str, dir: &PathBuf| 
    {
        // let a = PathBuf::from("D:\\work\\rust");
        // let b = fs:canonicalize("D:\\work\\rust").unwrap();
        // println!("runCmd :{}-{:?}-{}-{:?}", cmd,a,a.is_relative(),env::current_dir().unwrap());

        // #[cfg(target_os = "windows")]
        // {
        //     Command::new("cmd")
        //              .arg("/C")
        //              .arg("dir /W")
        //              .status()
        //              .expect("failed to execute process");
        // }
        println!("{:?}", cmd);
        if cfg!(target_os = "windows") {
            Command::new("cmd")
                .arg("/C")
                .arg(cmd)
                .current_dir(dir)
                .status()
                .expect("failed to execute process");
        } else {
            Command::new("sh")
                .current_dir(dir)
                .arg("-c")
                .arg(cmd)
                .status()
                .expect("failed to execute process");
        }

        return;
    };

    for i in cmdArray {
        runCmd(i.as_str(),dir);
    }
    
}
