use std::{collections::HashMap, fs, io::{BufRead, BufReader}};
use crate::af::*;

#[derive(Clone, Copy, PartialEq)]
pub enum Format {
    APX,
    CNF,
}

pub fn get_input(file_path : &str, mut format : Format) -> (ArgumentationFramework, Vec<String>, Format) {
    let fl = get_first_line(file_path);
    if fl.trim().starts_with("p af") {
        format = Format::CNF;
    }
    match format {
        Format::APX =>  {
            let res = reading_apx(file_path);
            (res.0, res.1, Format::APX)
        },
        Format::CNF => (reading_cnf(file_path), Vec::new(), Format::CNF),
    }
}
fn get_first_line(file_path : &str ) -> String {
    let file = match fs::File::open(&file_path) {
        Ok(file) => file,
        Err(_) => panic!("Unable to read title from {:?}", &file_path),
    };
    let mut buffer = BufReader::new(file);
    let mut first_line = String::new();
    let _ = buffer.read_line(&mut first_line);

    first_line
}
pub fn reading_cnf( file_path : &str) -> ArgumentationFramework {
    let contents = fs::read_to_string(file_path)
    .expect("Should have been able to read the file");
    let mut content_iter = contents.trim().split('\n');
    let first_line = content_iter.next().unwrap();
    let iter: Vec<&str> = first_line.split_ascii_whitespace().collect();
    let nb_arg = iter[2].parse::<usize>().unwrap();
    let mut af = ArgumentationFramework::new(nb_arg);
    for line in content_iter {
        if !line.starts_with('#') && (!line.trim().eq("")) {
            let (attacker,target) = parse_cnfattack_line(line);
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

fn parse_cnfattack_line (line : &str) -> (i32,i32) {
    let mut a = line.split_ascii_whitespace();
    let att = a.next().unwrap().parse::<i32>().unwrap();
    let targ = a.next().unwrap().parse::<i32>().unwrap();
    (att,targ)
}

pub fn reading_apx( file_path : &str) -> (ArgumentationFramework, Vec<String>) {
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