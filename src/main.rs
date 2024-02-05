use std::io::Write;
use std::io::Read;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::env;
use std::process::{Command, Stdio};
use std::str::FromStr;
use std::thread;
use std::thread::available_parallelism;
use std::sync::{RwLock, Arc};
use std::time::{Duration, Instant};
use colored::Colorize;
use wait_timeout::ChildExt;
use crate::grounded::*;
use crate::parser::*;

mod grounded;
mod af;
mod parser;

struct Job {
    file_path : PathBuf,
    step_arg : usize,
    grounded : Vec<Label>,
    nb_arg : usize,
    arg_names: Vec<String>,
    stop : bool,
}

fn create_data(job_lock : Arc<RwLock<Job>>, solver_path : PathBuf) {
    let r = job_lock.read().unwrap();
    let max_time = (4*3600) / r.nb_arg as u64;
    drop(r);
    loop {
        let mut r = job_lock.write().unwrap();
        if r.stop { break; }
        if r.nb_arg <= r.step_arg { break; }
        if r.grounded[r.step_arg] != Label::UNDEC {
            //eprintln!("{}","IN GROUNDED".green());
            (*r).step_arg +=1;
            drop(r);
            continue;
        }
        let arg_id = r.step_arg;
        r.step_arg+=1;
        let file_path  = r.file_path.clone();
        let arg_name = if r.arg_names.is_empty() { arg_id.to_string() }
        else { r.arg_names[arg_id].clone() };
        
        drop(r);
        
        let mut child = Command::new(solver_path.clone())
        .arg("solve")
        .arg("-p")
        .arg("DC-CO")
        .arg("-f")
        .arg(file_path)
        .arg("-r")
        .arg("apx")
        .arg("-a")
        .arg(arg_name)
        .arg("--logging-level")
        .arg("off")
        .stdout(Stdio::piped())
        //.stderr(Stdio::piped())
        .spawn()
        .unwrap();
    
        let one_sec = Duration::from_secs(max_time);
        let status_code = match child.wait_timeout(one_sec).unwrap() {
            Some(status) => status.code(),
            None => {
                // child hasn't exited yet
                child.kill().unwrap();
                child.wait().unwrap().code()
            }
        };

        if status_code != Some(0) {
            let mut r = job_lock.write().unwrap();
            r.stop = true;
            break;
        }
        let mut buf = Vec::new();
        //let mut buf_err = Vec::new();
        let _  = child.stdout.unwrap().read_to_end(&mut buf);
        //let _  = child.stderr.unwrap().read_to_end(&mut buf_err);
        //println!("{}", status_code.unwrap());
        let output = String::from_utf8(buf).unwrap();
        //let err = String::from_utf8(buf_err).unwrap();
        //println!("res : {}", output);
        //println!("err : {}", err);

        if output.starts_with("YES") {
            let mut r = job_lock.write().unwrap();
            r.grounded[arg_id] = Label::IN;
        }
        else if output.starts_with("NO") {
            let mut r = job_lock.write().unwrap();
            r.grounded[arg_id] = Label::OUT;
        }
        
        //println!("output : {}", String::from_utf8(child.stdout).unwrap().trim());

    }
}

fn main() {
    let default_parallelism_approx = available_parallelism().unwrap().get();
    let mut arg = env::args().skip(1);
    let data_folder = arg.next().unwrap();
    let solver_path = PathBuf::from_str(arg.next().unwrap().as_str()).unwrap();
    let dir = Path::new(data_folder.as_str());
    if !dir.is_dir() { return; }

    let _ = fs::create_dir("result");
    let all_file : Vec<PathBuf>  = fs::read_dir(dir).unwrap().filter_map(|mut entry| 
        if !entry.as_mut().unwrap().path().to_str().unwrap().ends_with(".arg") {
            return Some(entry.unwrap().path());
        }
        else {
            return None;
        }
    ).collect();
    //let file_lock_save = Arc::new(RwLock::new(all_file));
    for f in all_file {
        let mut res_path = PathBuf::from("result");
        let temp = f.clone();
        let file_name = temp.to_str().unwrap().split(&['\\', '/']);
        res_path = res_path.join(file_name.last().unwrap());
        println!("-----------------------------------------------");

        if res_path.exists() { 
            println!("{} {} ", f.display(), "already computed".yellow());
            continue; 
        }
        let (af, arg_name) = if f.ends_with("af") {
            get_input(f.to_str().unwrap(), Format::CNF)
        }
        else {
            get_input(f.to_str().unwrap(), Format::APX)
        };
        println!("{}", f.display());
        let job = Job{file_path : f, step_arg: 0, grounded : solve(&af), nb_arg : af.nb_argument, arg_names : arg_name, stop:false};
        let job_lock = Arc::new(RwLock::new(job));
        let mut thread_join_handle = Vec::with_capacity(default_parallelism_approx);
        for _ in 0..default_parallelism_approx {
            let solver_path1 = solver_path.clone();
            let job_lock1 = job_lock.clone();
            thread_join_handle.push(
                thread::spawn(move || {
                    create_data(job_lock1, solver_path1);
                })
            );
        }
        for t in thread_join_handle {
            let _ = t.join();
        }
        println!("{}", "done".green());
        let mut res = String::with_capacity(af.nb_argument);
        let w = job_lock.write().unwrap();
        if w.stop { drop(w); continue; }
        for (i, arg) in w.grounded.iter().enumerate() {
            if *arg == Label::IN {
                res.push_str(&i.to_string());
                res.push(',');
            }
        }

        let mut file = File::create(res_path).unwrap();
        file.write_all(res.as_bytes()).unwrap();
    }
    
}
