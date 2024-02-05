use std::{collections::HashMap, fs};
use crate::af::*;

pub enum Format {
    APX,
    CNF,
}

pub fn get_input(file_path : &str, format : Format) -> (ArgumentationFramework, Vec<String>) {
    match format {
        Format::APX => readingAPX(file_path),
        Format::CNF => (readingCNF(file_path), Vec::new()),
        //Format::CNF => readingCNF_perf(file_path),
    }
}

pub fn readingCNF( file_path : &str) -> ArgumentationFramework {
    let contents = fs::read_to_string(file_path)
    .expect("Should have been able to read the file");
    let mut content_iter = contents.trim().split('\n');
    let first_line = content_iter.next().unwrap();
    let iter: Vec<&str> = first_line.split_ascii_whitespace().collect();
    let nb_arg = iter[2].parse::<usize>().unwrap();
    let mut af = ArgumentationFramework::new(nb_arg);
    for line in content_iter {
        if !line.starts_with('#') && (!line.trim().eq("")) {
            let (attacker,target) = parseCNFAttackLine(line);
            af.add_attack(attacker, target);
        }
    }
    af
}

fn find_number_argument(file_path : &str) -> i32 {
    let contents = fs::read_to_string(file_path)
        .expect("Should have been able to read the file");
    let a = contents.trim().split('\n');
    let mut nb_arg = 0;
    for line in a {
        if line.starts_with("arg") { nb_arg +=1; }
        else { break; }
    }
    nb_arg
}

fn parseCNFAttackLine (line : &str) -> (i32,i32) {
    let mut a = line.split_ascii_whitespace();
    let att = a.next().unwrap().parse::<i32>().unwrap();
    let targ = a.next().unwrap().parse::<i32>().unwrap();
    (att,targ)
}
fn parseAPXAttackLine (line : &str) -> (i32,i32) {
    let buff = line.strip_prefix("att(").unwrap();
    let buff2 = buff.strip_suffix(").").unwrap();
    let buff2 = buff2.replace(",", " ");

    parseCNFAttackLine(&buff2)
}

pub fn readingAPX( file_path : &str) -> (ArgumentationFramework, Vec<String>) {
    let mut index = 1;
    let mut index_map = HashMap::new();
    let nb_arg = find_number_argument(file_path);
    let mut af = ArgumentationFramework::new(nb_arg as usize);

    let contents = fs::read_to_string(file_path)
        .expect("Should have been able to read the file");
    let a = contents.trim().split('\n');
    let mut arg_names = Vec::new();
    for line in a {
        if !line.starts_with('#') && (!line.trim().eq("")) {
            if line.starts_with("arg") {
                let buff = line.strip_prefix("arg(").unwrap();
                let buff2 = buff.strip_suffix(").").unwrap();
                arg_names.push(String::from(buff2));
                index_map.insert(buff2, index);
                index+=1;
                continue;
            }
            if line.starts_with("att") {
                let buff = line.strip_prefix("att(").unwrap();
                let buff2 = buff.strip_suffix(").").unwrap();
                let buff2 = buff2.replace(",", " ");
                let mut s = buff2.split(" ");
                let att = *index_map.get(s.next().unwrap()).unwrap();
                let target = *index_map.get(s.next().unwrap()).unwrap();
                af.add_attack(att, target);
            }
        }
    }
    
    (af, arg_names)
}