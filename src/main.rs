use std::io::{self, stdout, Write};
use std::fs::{self, DirEntry, File};
use std::path::{Path, PathBuf};
use std::env;
use std::process::Command;
use std::str::FromStr;
use std::thread;
use std::thread::available_parallelism;
use std::sync::{RwLock, Arc};
use rand::Rng;
use rand::distributions::{Distribution, Uniform};

fn find_number_arg(file_name : &PathBuf) -> u32 {
    let mut n = 0;
    let file = fs::read_to_string(file_name).unwrap();
    for line in file.lines() {
        if line.starts_with("arg") || line.starts_with("?arg") {
            n+=1;
        }
        else {
            break;
        }
    }
    n
}

fn create_data(lock_file : Arc<RwLock<Vec<PathBuf>>>, file_output : Arc<RwLock<File>>, solver_path : PathBuf) {
    loop {
    
        let mut nb_yes = 0;
        let mut nb_no  = 0;
        let mut queue = lock_file.write().unwrap();
        if (*queue).is_empty() { break; }
        let file_name = (*queue).pop().unwrap();
        drop(queue);
        println!("{}", file_name.display());
        let nb_arg = find_number_arg(&file_name);
        let mut rng = rand::thread_rng();
        let mut already  = Vec::new();
        while nb_yes < 2 && nb_no < 2 {
            let rand = rng.gen_range(0..nb_arg);
            let taeydennae =  Command::new(solver_path.clone())
            .arg("-p")
            .arg("PCA-CO")
            .arg("-f")
            .arg(file_name.clone())
            .arg("-a")
            .arg(rand.to_string()).output().unwrap();
            let out = String::from_utf8(taeydennae.stdout).unwrap();
            println!("{:?} {}", out, rand);
            if out.starts_with("YES") {
                nb_yes +=1;
            }
            else if out.starts_with("NO"){
                nb_no +=1;
            }
            already.push(rand);
            //break;
        }
    }
}

fn main() {
    let default_parallelism_approx = available_parallelism().unwrap().get();
    let mut arg = env::args().skip(1);
    let data_folder = arg.next().unwrap();
    let solver_path = PathBuf::from_str(arg.next().unwrap().as_str()).unwrap();
    let dir = Path::new(data_folder.as_str());
    if !dir.is_dir() { return; }
    let f = File::create("dataset").unwrap();
    let all_file : Vec<PathBuf>  = fs::read_dir(dir).unwrap().filter_map(|mut entry| 
        if entry.as_mut().unwrap().path().to_str().unwrap().ends_with(".apx") {
            return Some(entry.unwrap().path());
        }
        else {
            return None;
        }
    ).collect();
    let file_lock_save = Arc::new(RwLock::new(all_file));
    let dataset_lock_save = Arc::new(RwLock::new(f));
    let mut thread_join_handle = Vec::new();

    for _ in 0..default_parallelism_approx {
        let file_lock = file_lock_save.clone();
        let dataset_lock = dataset_lock_save.clone();
        let solver_path1 = solver_path.clone();
        thread_join_handle.push(
            thread::spawn(move || {
                create_data(file_lock, dataset_lock, solver_path1);
            })
        );
    }
    for t in thread_join_handle {
        let _ = t.join();
    }
    

}
