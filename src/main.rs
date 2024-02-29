use std::cmp::max;
use std::io::Write;
use std::io::Read;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::env;
use std::process::exit;
use std::process::ExitCode;
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
    error : bool,
    file_type : Format,
}

fn create_data(job_lock : Arc<RwLock<Job>>, solver_path : PathBuf, problem_type : String) {
    let r = job_lock.read().unwrap();
    let max_time = (4*3600) / r.nb_arg as u64;
    let format = r.file_type.clone();
    let kissat = true;
    drop(r);
    loop {
        let mut r = job_lock.write().unwrap();
        if r.stop { break; }
        if r.nb_arg <= r.step_arg { break; }
        if r.grounded[r.step_arg] != Label::UNDEC {
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
        let t = solver_path.clone();
        let mut cmd = Command::new(t);
        cmd.arg("solve")
            .arg("-p")
            .arg(&problem_type)
            .arg("-f")
            .arg(file_path)   
            .arg("-a")
            .arg(arg_name)
            .arg("--logging-level")
            .arg("off");
        if format == Format::APX {
            cmd.arg("-r").arg("apx");
        }
        if kissat {
            cmd.arg("--external-sat-solver")
            .arg("../kissat/build/kissat");
        }
        let mut child = cmd.stdout(Stdio::piped())
            .spawn()
            .unwrap();
        
        //let start = Instant::now();
        let mut timeout = false;
        let one_sec = Duration::from_secs(max_time);
        let status_code = match child.wait_timeout(one_sec).unwrap() {
            Some(status) => status.code(),
            None => {
                // child hasn't exited yet
                child.kill().unwrap();
                timeout = true;
                //println!("{} {}", max_time, start.elapsed().as_secs());
                child.wait().unwrap().code()
            }
        };
        
        let mut buf = Vec::new();
        //let mut buf_err = Vec::new();
        let _  = child.stdout.unwrap().read_to_end(&mut buf);
        //let _  = child.stderr.unwrap().read_to_end(&mut buf_err);
        //println!("{}", status_code.unwrap());
        let output = String::from_utf8(buf).unwrap();
        //let err = String::from_utf8(buf_err).unwrap();
        //println!("res : {}", output);
        //println!("err : {}", err);
        if status_code != Some(0) {
            //println!("solve -p DC-CO -f {} -r apx -a {} --logging-level off",file_path2.display() , arg_name2);
            //println!("{}", status_code.unwrap());
            let mut r = job_lock.write().unwrap();
            r.stop = true;
            r.error = !timeout;
            break;
        }
        
        if output.starts_with("YES") {
            let mut r = job_lock.write().unwrap();
            r.grounded[arg_id] = Label::IN;
        }
        else if output.starts_with("NO") {
            let mut r = job_lock.write().unwrap();
            r.grounded[arg_id] = Label::OUT;
        }
        //println!("output : {}", String::from_utf8(child.stdout).unwrap().trim())
    }
}

fn main() {
    let default_parallelism_approx = available_parallelism().unwrap().get();
    let mut arg = env::args().skip(1);
    let data_folder = arg.next().unwrap();
    let solver_path = PathBuf::from_str(arg.next().unwrap().as_str()).unwrap();
    let problem_type = arg.next().unwrap();
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
    let mut dir_name = String::from("result_");
    dir_name.push_str(&problem_type);
    let res_path_org = PathBuf::from(dir_name);
    if !res_path_org.exists() {
        fs::create_dir(&res_path_org).unwrap();
    }
    for f in all_file {
        let mut res_path = res_path_org.clone();
        let temp = f.clone();
        let file_name = temp.to_str().unwrap().split(&['\\', '/']);
        res_path = res_path.join(file_name.last().unwrap());
        println!("{}", res_path.display());
        println!("-----------------------------------------------");

        if res_path.exists() {
            println!("{} {} ", f.display(), "already computed".cyan());
            continue; 
        }
        let (af, arg_name, format) = if f.ends_with("af") {
            get_input(f.to_str().unwrap(), Format::CNF)
        }
        else {
            get_input(f.to_str().unwrap(), Format::APX)
        };
        println!("{}", f.display());
        let job = Job{file_path : f, step_arg: 0, grounded : solve(&af), nb_arg : af.nb_argument, arg_names : arg_name, stop:false, error : false, file_type : format};
        let job_lock = Arc::new(RwLock::new(job));
        let mut thread_join_handle = Vec::with_capacity(default_parallelism_approx);
        for _ in 0..default_parallelism_approx {
            let solver_path1 = solver_path.clone();
            let job_lock1 = job_lock.clone();
            let problem_type1 = problem_type.clone();
            thread_join_handle.push(
                thread::spawn(move || {
                    create_data(job_lock1, solver_path1, problem_type1);
                })
            );
        }
        for t in thread_join_handle {
            let _ = t.join();
        }
        let w = job_lock.write().unwrap();

        if w.stop && !w.error { println!("{}", "Take Too much time".yellow()); }
        else if w.stop && w.error { println!("{}", "Error from solver".red()); }
        else { println!("{}", "done".green()); }

        let mut res = String::with_capacity(af.nb_argument);
        if w.stop { drop(w); continue; }
        for (i, arg) in w.grounded.iter().enumerate() {
            if *arg == Label::IN {
                res.push_str(&i.to_string());
                res.push(',');
            }
        }
        let mut file = File::create(res_path).unwrap();
        file.write_all(res.as_bytes()).unwrap();
        let _ = file.flush();
    }
    
}
