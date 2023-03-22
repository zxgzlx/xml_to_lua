use clap::{App, Arg};
use lazy_static::lazy_static;
use std::io::prelude::*;
use std::sync::Mutex;
use std::{fs::File, path::PathBuf};
use xml::reader::{EventReader, XmlEvent};
use std::io::{self, Write};

lazy_static! {
    static ref INPUT_PATH: Mutex<String> = Mutex::new(String::new());
    static ref OUT_PATH: Mutex<String> = Mutex::new(String::new());
}

fn main() -> std::io::Result<()> {
    // command_arg_parse();
    command_arg_user_input();

    // 遍历文件夹下面所有的xml文件
    find_xml_file(INPUT_PATH.lock().as_deref().unwrap())?;
    Ok(())
}

// 命令行参数直接传入方式交互
fn command_arg_parse() {
    let matches = App::new("MyApp")
        .arg(
            Arg::with_name("input")
                .help("Sets the input path to use")
                .required(true)
                .takes_value(true)
                .index(1),
        )
        .arg(
            Arg::with_name("output")
                .help("Sets the out path to use")
                .required(true)
                .takes_value(true)
                .index(2),
        )
        .get_matches();
    let input = matches.value_of("input").unwrap();
    let output = matches.value_of("output").unwrap();
    INPUT_PATH.lock().unwrap().push_str(input);
    OUT_PATH.lock().unwrap().push_str(output);
    println!(
        "The path you entered is: {} output path {}",
        INPUT_PATH.lock().as_deref().unwrap(),
        OUT_PATH.lock().as_deref().unwrap()
    );
}

// 用户通过应用交互输入
fn command_arg_user_input() {
    loop {
        println!("请输入xml目录: ");
        io::stdout().flush().unwrap();
        print!("> ");
        io::stdout().flush().unwrap();
    
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
    
        let input = input.trim();
    
        match input {
            "exit" => {
                println!("退出");
                break;
            }
            "" => {
                println!("输入目录为空");
                continue;
            }
            _ => {
                // do something with the input
                println!("输入含有xml目录: {}", input);
                INPUT_PATH.lock().unwrap().push_str(input);
                break;
            }
        }
    }

    loop {
        println!("\n输入最终输出的lua目录: ");
        io::stdout().flush().unwrap();
        print!("> ");
        io::stdout().flush().unwrap();
    
        let mut output = String::new();
        io::stdin().read_line(&mut output).unwrap();
    
        let output = output.trim();
    
        match output {
            "exit" => {
                println!("退出");
                break;
            }
            "" => {
                println!("输出lua目录为空");
                continue;
            }
            _ => {
                // do something with the input
                println!("输出lua目录: {}", output);
                OUT_PATH.lock().unwrap().push_str(output);
                break;
            }
        }
    }
}

fn find_xml_file(file_path: &str) -> std::io::Result<()> {
    let mut files = std::fs::read_dir(file_path)?;
    while let Some(file) = files.next() {
        let file = file?;
        let path = file.path();
        if (path.is_dir()) {
            // println!("path: {}", path.display());
            // get dir name
            let dir_name = path.file_name().unwrap().to_str().unwrap();
            println!("dir name: {}", dir_name);
            find_xml_file(path.to_str().unwrap())?;
            continue;
        }
        if path.extension().unwrap() == "xml" {
            println!("path: {}", path.display());
            convert_xml_to_lua(path);
        }
    }

    Ok(())
}

fn indent(size: usize) -> String {
    const INDENT: &'static str = "    ";
    (0..size)
        .map(|_| INDENT)
        .fold(String::with_capacity(size * INDENT.len()), |r, s| r + s)
}

fn convert_xml_to_lua(path: PathBuf) -> std::io::Result<()> {
    // 打开XML文件
    let file = File::open(path.as_path())?;
    let reader = EventReader::new(file);
    let file_name_str = path
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .split('.')
        .next()
        .unwrap();

    // 转换为Lua格式的字符串
    let mut depth = 0;
    let mut lua_str = "local M = {\n".to_owned();
    for event in reader {
        match event {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                // attributes判断是否有属性
                if attributes.len() != 0 {
                    lua_str += &format!("{}{}", indent(depth), "{");
                    for attr in attributes {
                        lua_str += &format!("{}=\"{}\",", attr.name.local_name, attr.value);
                    }
                    lua_str += "},\n";
                }

                depth += 1;
            }
            Ok(XmlEvent::EndElement { name }) => {
                depth -= 1;
            }
            Err(e) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ))
            }
            _ => (),
        }
    }
    lua_str += "}\n";
    lua_str += &format!("_G.ConfigTable.{} = M", file_name_str);
    println!("文件夹名字：{}", file_name_str);

    // 将字符串保存到Lua文件
    // check if the directory exists, if not, create it
    // if !PathBuf::from("config_lua").exists() {
    //     std::fs::create_dir("config_lua")?;
    // }
    let mut file = File::create(format!(
        "{}/{}.lua",
        OUT_PATH.lock().as_deref().unwrap(),
        file_name_str
    ))?;
    file.write_all(lua_str.as_bytes())?;

    Ok(())
}
