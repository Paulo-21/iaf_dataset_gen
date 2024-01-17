use std::io;
use std::fs::{self, DirEntry, File};
use std::path::{Path, PathBuf};
use std::env;
use std::process::Command;
use std::thread;
use std::thread::available_parallelism;
use std::sync::{RwLock, Arc};


fn create_data(lock_file : Arc<RwLock<Vec<PathBuf>>>, file_output : Arc<RwLock<File>>) {
    let mut queue = lock_file.write().unwrap();
    if (*queue).is_empty() {
        return;
    }
    let file_name = (*queue).pop().unwrap();
    let taeydennae =  Command::new("./taeydennae_linux_x86-64")
        .arg("-p")
        .arg("PCA-CO")
        .arg("-f")
        .arg(file_name)
        .arg("-a")
        .arg("1").output();
    println!("{:?}", taeydennae);
}

fn main() {
    let default_parallelism_approx = available_parallelism().unwrap().get();
    let mut arg = env::args();
    let _ = arg.next();
    let data_folder = arg.next().unwrap();
    let solver_path = arg.next().unwrap();
    let dir = Path::new(data_folder.as_str());
    println!("{:?}", dir);
    let mut f = File::create("dataset").unwrap();
    if !dir.is_dir() { return; }
    let all_file : Vec<PathBuf>  = fs::read_dir(dir).unwrap().map(|entry| entry.unwrap().path()).collect();
    let file_lock_save = Arc::new(RwLock::new(all_file));
    let dataset_lock_save = Arc::new(RwLock::new(f));
    let mut thread_join_handle = Vec::new();
    for _ in 0..default_parallelism_approx {
        let file_lock = file_lock_save.clone();
        let dataset_lock = dataset_lock_save.clone();
        thread_join_handle.push(
            thread::spawn(move || {
                create_data(file_lock, dataset_lock);
            })
        );
    }
    for t in thread_join_handle {
        let _ = t.join();
    }
    

}
