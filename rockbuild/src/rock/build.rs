#![allow(non_snake_case)]
use std::io::{Result,Error,ErrorKind};
use std::path::{PathBuf};
// use super::genv;
use crate::rock::misc;
use serde_json::Value;
use std::collections::HashMap;
use std::fs::{metadata};
use std::process::Child;
// use std::thread;
// use std::time::Duration;
use std::env;

fn cmpFileTime(a:&PathBuf,b:&PathBuf)->bool
{
    let amd = metadata(a).unwrap().modified().unwrap();
    let bmd = metadata(b).unwrap().modified().unwrap();

    return amd <= bmd;
}

fn buildObjectFile(opt: &mut misc::OptionData) -> Result<Option<Child>> {
    let mut obj:PathBuf;
    let src:PathBuf;
    let mut cross:PathBuf;
    let mut cmd:String;
    let mut args:String = String::new();    

    if !opt.ASMSource.is_empty()
    {
        src = opt.ASMSource.pop().unwrap();
        obj = src.clone();
        obj.set_extension("o");

        if PathBuf::from(&opt.CROSSCOMPILE).is_dir()
        {
            cross = PathBuf::from(&opt.CROSSCOMPILE);
            cross.push(&opt.AS);
            cmd = String::from(cross.to_str().unwrap());
        }
        else
        {
            cmd = opt.CROSSCOMPILE.clone();
            cmd.push_str(opt.AS.as_str());
        }

        args.push_str("-c");

        if opt.Type == "ShareLib"
        {
            args.push(' ');
            args.push_str("-fPIC");
        }
        
        args.push(' ');
        args.push_str(src.to_str().unwrap());
        
        args.push(' ');
        args.push_str("-o");
        
        args.push(' ');
        args.push_str(obj.to_str().unwrap());
        
        if !opt.AFLAGS.is_empty()
        {
            args.push(' ');
            args.push_str(opt.AFLAGS.as_str());
        }
        if !opt.DEFS.is_empty()
        {
            args.push(' ');
            args.push_str(opt.DEFS.as_str());
        }
        
        if !opt.INCLUDES.is_empty()
        {
            args.push(' ');
            args.push_str(opt.INCLUDES.as_str());
        }
    }
    else if !opt.CSource.is_empty()
    {
        src = opt.CSource.pop().unwrap();
        obj = src.clone();
        obj.set_extension("o");

        if PathBuf::from(&opt.CROSSCOMPILE).is_dir()
        {
            cross = PathBuf::from(&opt.CROSSCOMPILE);
            cross.push(&opt.CC);
            cmd = String::from(cross.to_str().unwrap());
        }
        else
        {
            cmd = opt.CROSSCOMPILE.clone();
            cmd.push_str(opt.CC.as_str());
        }

        args.push_str("-c");

        if opt.Type == "ShareLib"
        {
            args.push(' ');
            args.push_str("-fPIC");
        }

        args.push(' ');
        args.push_str(src.to_str().unwrap());
        
        args.push(' ');
        args.push_str("-o");
        
        args.push(' ');
        args.push_str(obj.to_str().unwrap());
        
        if !opt.CFLAGS.is_empty()
        {
            args.push(' ');
            args.push_str(opt.CFLAGS.as_str());
        }

        if !opt.DEFS.is_empty()
        {
            args.push(' ');
            args.push_str(opt.DEFS.as_str());
        }
        if !opt.LIBPATH.is_empty()
        {
            args.push(' ');
            args.push_str(opt.LIBPATH.as_str());
        }
        if !opt.INCLUDES.is_empty()
        {
            args.push(' ');
            args.push_str(opt.INCLUDES.as_str());
        }
    }
    else if !opt.CXXSource.is_empty()
    {
        src = opt.CXXSource.pop().unwrap();
        obj = src.clone();
        obj.set_extension("o");

        if PathBuf::from(&opt.CROSSCOMPILE).is_dir()
        {
            cross = PathBuf::from(&opt.CROSSCOMPILE);
            cross.push(&opt.CXX);
            cmd = String::from(cross.to_str().unwrap());
        }
        else
        {
            cmd = opt.CROSSCOMPILE.clone();
            cmd.push_str(opt.CXX.as_str());
        }

        args.push_str("-c");

        if opt.Type == "ShareLib"
        {
            args.push(' ');
            args.push_str("-fPIC");
        }

        args.push(' ');
        args.push_str(src.to_str().unwrap());
        
        args.push(' ');
        args.push_str("-o");
        
        args.push(' ');
        args.push_str(obj.to_str().unwrap());
        
        if !opt.CXXFLAGS.is_empty()
        {
            args.push(' ');
            args.push_str(opt.CXXFLAGS.as_str());
        }

        if !opt.DEFS.is_empty()
        {
            args.push(' ');
            args.push_str(opt.DEFS.as_str());
        }
        if !opt.INCLUDES.is_empty()
        {
            args.push(' ');
            args.push_str(opt.INCLUDES.as_str());
        }
    }
    else 
    {
        // return Ok(String::from("Ok"));
        return Err(Error::from(ErrorKind::NotFound));
    }  

    opt.Object.push(obj.clone());
 
    if (opt.Rebuild == true) || (obj.exists()==false) || cmpFileTime(&obj,&src)
    {
        // println!("{:?} {:?}", cmd.as_str(),args);
        match misc::execAsync(cmd.as_str(),args,opt.IsSilent)
        {
            Ok(x) =>{return Ok(Some(x))}
            Err(e) =>{panic!("building {:?} error {:?}",src,e);}
        }
        // misc::execSync(cmd.as_str(),args);
        // buildObjectFile(opt)?;
    }

    return Ok(None);
    // return Err(Error::from(ErrorKind::NotFound));
    // return Ok(String::from("Ok"));
}

fn buildObject(opt: &mut misc::OptionData) -> Result<String> {
    let mut p:Vec<Child> = vec![];

    loop
    {
        if (p.len() as u64) < opt.Jobs
        {
            match buildObjectFile(opt)
            {
                // building
                Ok(Some(x)) =>{
                    p.push(x);
                    continue;
                }
                // no building
                Ok(None) =>{
                    continue;
                }
                // no file
                Err(_e) =>{
                    // break;
                }
            }
        }

        if p.is_empty()
        {
            break;
        }

        for _ in 0..opt.Jobs {
            if p.is_empty()
            {
                break;
            }

            let mut v = p.pop().unwrap();

            match v.try_wait()
            {
                Ok(Some(status)) =>{
                    if !status.success()
                    {
                        return Err(Error::from(ErrorKind::Other));
                    }
                }
                Ok(None) =>{
                    p.push(v);
                }
                Err(e) =>{
                    println!("{:?}", e);
                }
            }
        }
    }

    // buildObjectFile(opt)?;

    return Ok(String::from("Ok"));
}

fn buildProgram(opt: &misc::OptionData) -> Result<String> {
    let mut cross:PathBuf;
    let mut args:String = String::new();
    let mut cmd:String;
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
                targetExt.push_str(".exe");
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
                targetExt.push_str(".out");
            }
        }
    }

    if PathBuf::from(&opt.CROSSCOMPILE).is_dir()
    {
        cross = PathBuf::from(&opt.CROSSCOMPILE);
        cross.push(&opt.LD);
        cmd = String::from(cross.to_str().unwrap());
    }
    else
    {
        cmd = opt.CROSSCOMPILE.clone();
        cmd.push_str(opt.LD.as_str());
    }

    args.push_str("-o");
   
    args.push(' ');
    args.push_str(targetExt.as_str());

    for i in &opt.Object{
        if !args.is_empty()
        {
            args.push(' ');
        }
        args.push_str(i.to_str().unwrap());
    }

    for i in &opt.DependObject{
        if !args.is_empty()
        {
            args.push(' ');
        }
        args.push_str(i.to_str().unwrap());
    }

    if !opt.LDFLAGS.is_empty()
    {
        args.push(' ');
        args.push_str(opt.LDFLAGS.as_str());
    }

    if opt.IsMap
    {
        let map = format!(" -Wl,-Map={}.map",opt.Target);
        args.push_str(map.as_str());
    }
    
    if !opt.LIBPATH.is_empty()
    {
        args.push(' ');
        args.push_str(opt.LIBPATH.as_str());
    }
    
    if !opt.LIBS.is_empty()
    {
        args.push(' ');
        args.push_str(opt.LIBS.as_str());
    }

    misc::execSync(cmd.as_str(),args,opt.IsSilent)?;

    if opt.IsAsm
    {
        let tcmd;

        if PathBuf::from(&opt.CROSSCOMPILE).is_dir()
        {
            cross = PathBuf::from(&opt.CROSSCOMPILE);
            cross.push(&opt.OBJDUMP);
            tcmd = format!("{} -D -S {} > {}.asm",cross.to_str().unwrap(),target,opt.Target);
        }
        else
        {
            tcmd = format!("{}{} -D -S {} > {}.asm",opt.CROSSCOMPILE.clone(),opt.OBJDUMP.as_str(),targetExt,target);
        }

        misc::cmdSync(&vec![tcmd],&env::current_dir().unwrap());
    }

    if opt.IsStrip
    {
        let mut tcmd;

        if PathBuf::from(&opt.CROSSCOMPILE).is_dir()
        {
            cross = PathBuf::from(&opt.CROSSCOMPILE);
            cross.push(&opt.STRIP);
            tcmd = String::from(cross.to_str().unwrap());
        }
        else
        {
            tcmd = opt.CROSSCOMPILE.clone();
            tcmd.push_str(opt.STRIP.as_str());
        }

        let targs:String = format!("{}",targetExt);
        misc::execSync(tcmd.as_str(),targs,opt.IsSilent)?;
    }

    if opt.IsBinary
    {
        let mut tcmd;

        if PathBuf::from(&opt.CROSSCOMPILE).is_dir()
        {
            cross = PathBuf::from(&opt.CROSSCOMPILE);
            cross.push(&opt.OBJCOPY);
            tcmd = String::from(cross.to_str().unwrap());
        }
        else
        {
            tcmd = opt.CROSSCOMPILE.clone();
            tcmd.push_str(opt.OBJCOPY.as_str());
        }

        let targs:String = format!("-O binary {} {}.bin",targetExt,target);
        misc::execSync(tcmd.as_str(),targs,opt.IsSilent)?;
    }

    return Ok(String::from("Ok"));
}

fn buildStaticLib(opt: &misc::OptionData) -> Result<String> 
{
    let mut cross:PathBuf;
    let mut args:String = String::new();
    let mut cmd:String;
    // let mut target:String;
    let mut targetExt:String;

    targetExt = opt.Target.clone();
    // target = opt.Target.clone();
    if cfg!(target_os = "windows")
    {
        match PathBuf::from(&opt.Target).extension()
        {
            Some(_x) =>{
                // target = target.replace(&format!(".{:?}",x), "");
            }
            None =>{
                targetExt.push_str(".a");
            }
        }
    }
    else
    {
        match PathBuf::from(&opt.Target).extension()
        {
            Some(_x) =>{
                // target = target.replace(&format!(".{:?}",x), "");
            }
            None =>{
                targetExt.push_str(".a");
            }
        }
    }

    if PathBuf::from(&opt.CROSSCOMPILE).is_dir()
    {
        cross = PathBuf::from(&opt.CROSSCOMPILE);
        cross.push(&opt.AR);
        cmd = String::from(cross.to_str().unwrap());
    }
    else
    {
        cmd = opt.CROSSCOMPILE.clone();
        cmd.push_str(opt.AR.as_str());
    }

    args.push_str("r");
   
    args.push(' ');
    args.push_str(targetExt.as_str());

    for i in &opt.Object{
        if !args.is_empty()
        {
            args.push(' ');
        }
        args.push_str(i.to_str().unwrap());
    }

    for i in &opt.DependObject{
        if !args.is_empty()
        {
            args.push(' ');
        }
        args.push_str(i.to_str().unwrap());
    }

    if !opt.LIBPATH.is_empty()
    {
        args.push(' ');
        args.push_str(opt.LIBPATH.as_str());
    }
    
    if !opt.LIBS.is_empty()
    {
        args.push(' ');
        args.push_str(opt.LIBS.as_str());
    }

    misc::execSync(cmd.as_str(),args,opt.IsSilent)?;

    return Ok(String::from("Ok"));
}

fn buildSharelib(opt: &misc::OptionData) -> Result<String> 
{
    let mut cross:PathBuf;
    let mut args:String = String::new();
    let mut cmd:String;
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
                targetExt.push_str(".dll");
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
                targetExt.push_str(".so");
            }
        }
    }

    if PathBuf::from(&opt.CROSSCOMPILE).is_dir()
    {
        cross = PathBuf::from(&opt.CROSSCOMPILE);
        cross.push(&opt.LD);
        cmd = String::from(cross.to_str().unwrap());
    }
    else
    {
        cmd = opt.CROSSCOMPILE.clone();
        cmd.push_str(opt.LD.as_str());
    }

    args.push_str("-shared");
    args.push(' ');
    args.push_str("-o");
   
    args.push(' ');
    args.push_str(targetExt.as_str());

    for i in &opt.Object{
        if !args.is_empty()
        {
            args.push(' ');
        }
        args.push_str(i.to_str().unwrap());
    }

    for i in &opt.DependObject{
        if !args.is_empty()
        {
            args.push(' ');
        }
        args.push_str(i.to_str().unwrap());
    }

    if !opt.LDFLAGS.is_empty()
    {
        args.push(' ');
        args.push_str(opt.LDFLAGS.as_str());
    }

    if opt.IsMap
    {
        let map = format!(" -Wl,-Map={}.map",target);
        args.push_str(map.as_str());
    }
    
    if !opt.LIBPATH.is_empty()
    {
        args.push(' ');
        args.push_str(opt.LIBPATH.as_str());
    }
    
    if !opt.LIBS.is_empty()
    {
        args.push(' ');
        args.push_str(opt.LIBS.as_str());
    }

    misc::execSync(cmd.as_str(),args,opt.IsSilent)?;

    if opt.IsAsm
    {
        let tcmd;

        if PathBuf::from(&opt.CROSSCOMPILE).is_dir()
        {
            cross = PathBuf::from(&opt.CROSSCOMPILE);
            cross.push(&opt.OBJDUMP);
            tcmd = format!("{} -D -S {} > {}.asm",cross.to_str().unwrap(),targetExt,target);
        }
        else
        {
            tcmd = format!("{}{} -D -S {} > {}.asm",opt.CROSSCOMPILE.clone(),opt.OBJDUMP.as_str(),targetExt,target);
        }

        misc::cmdSync(&vec![tcmd],&env::current_dir().unwrap());
    }

    if opt.IsStrip
    {
        let mut tcmd;

        if PathBuf::from(&opt.CROSSCOMPILE).is_dir()
        {
            cross = PathBuf::from(&opt.CROSSCOMPILE);
            cross.push(&opt.STRIP);
            tcmd = String::from(cross.to_str().unwrap());
        }
        else
        {
            tcmd = opt.CROSSCOMPILE.clone();
            tcmd.push_str(opt.STRIP.as_str());
        }

        let targs:String = format!("{}",targetExt);
        misc::execSync(tcmd.as_str(),targs,opt.IsSilent)?;
    }

    return Ok(String::from("Ok"));
}


fn buildOption(optName: &PathBuf, fileMap: &HashMap<PathBuf, Value>) -> Result<String> {
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
                                buildOption(&PathBuf::from(s), fileMap)?;
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
    let mut option: misc::OptionData = misc::expandOption(&fileJson);

    if !option.IsSilent
    {
        println!("Building {:?}", optName);
        println!("{:?}", option);
    }

    match option.Type.as_str() {
        "Program" => {
            buildObject(&mut option)?;
            buildProgram(&option)?;
        }
        "StaticLib" => {
            buildObject(&mut option)?;
            buildStaticLib(&option)?;
        }
        "ShareLib" => {
            buildObject(&mut option)?;
            buildSharelib(&option)?;
        }
        "Object" => {
            buildObject(&mut option)?;
        }
        __ => {
            panic!("Type No Support!");
        }
    }

    misc::cmdSync(&misc::valueToVec(&fileJson["CmdStop"]), &fileAbsDir);

    return Ok(String::from("Ok"));
}

pub fn building(rockfile: PathBuf, rebuild: bool) -> Result<String> {
    // let buildDir = rockfile.parent().unwrap().canonicalize().unwrap();

    // misc::cdDir(buildDir);

    // let file = File::open(rockfile)?;
    // let reader = BufReader::new(file);
    // let rootfileJson:Value = serde_json::from_reader(reader)?;
    // let mut data:misc::OptionData = misc::getOptionFromJson(&rootfileJson);

    // data.Rebuild = rebuild;
    // println!("{:?}",data);

    // println!("getAbsPath:  {:?}", misc::getAbsPath(&rockfile));
    // println!("getAbsPath:  {:?}", misc::getAbsPath(&PathBuf::from("\\\\?\\D:\\work\\rust\\demo\\rock.json")));
    // println!("getAbsPath:  {:?}", misc::getAbsPath(&PathBuf::from("D:\\work\\rust\\demo\\rock.json")));
    // println!("getAbsPath:  {:?}", misc::getAbsDir(&PathBuf::from("..\\demo\\rock.json")));

    let fileAbsPath = misc::getAbsPath(&rockfile);
    let mut fileMap = HashMap::new();

    misc::getOption(&fileAbsPath, &mut fileMap);

    // println!("{:?}", fileMap);

    misc::initEnvs(
        &fileMap.get(&fileAbsPath).unwrap(),
        rebuild,
        &misc::getAbsDir(&rockfile),
    );
    buildOption(&fileAbsPath, &mut fileMap)?;

    return Ok(String::from("Ok"));
}
