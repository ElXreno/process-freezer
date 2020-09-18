/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use nix::unistd::Pid;
use nix::sys::signal::{self, Signal};

fn main() {
    let mut sleeping = false;

    loop {    
        let processes = procfs::process::all_processes().unwrap();
        let mut to_process: Vec<procfs::process::Process> = vec![];

        for process in processes {
            match process.stat.comm.as_str() {
                "java" | "javac" => {            
                    if process.stat.state == 'Z' {
                        continue;
                    }
                    
                    to_process.push(process);
                },
                _ => {}
            }
        }
        
        if procfs::Meminfo::new().unwrap().mem_available.unwrap() >= 4294967296 {
            if !sleeping {
                println!("Mem available, unfreezing all processes and sleeping");
                sleeping = true;
            }
            for process in &to_process {
                if process.stat.state != 'S' {
                    signal::kill(Pid::from_raw(process.stat.pid), Signal::SIGCONT).unwrap();
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(1500));
            continue;
        } else {
            sleeping = false;
        }
        
        match to_process.len() {
            0 => {
                std::thread::sleep(std::time::Duration::from_millis(500));
            },
            1 => {
                signal::kill(Pid::from_raw(to_process.first().unwrap().stat.pid), Signal::SIGCONT).unwrap();
                
                std::thread::sleep(std::time::Duration::from_millis(500));
            },
            _ => {
                let max = to_process.iter().max_by(|x, y| x.status().unwrap().vmrss.unwrap().cmp(&y.status().unwrap().vmrss.unwrap())).unwrap();
                
                for process in &to_process {
                    if process.stat.pid != max.stat.pid && process.stat.state != 'T'{
                        println!("Freezing {}", process.stat.pid);
                        signal::kill(Pid::from_raw(process.stat.pid), Signal::SIGSTOP).unwrap();
                    }
                }
                
                if max.stat.state != 'S' {
                    println!("Unfreezing {}", max.stat.pid);
                    signal::kill(Pid::from_raw(max.stat.pid), Signal::SIGCONT).unwrap();
                }
                
                std::thread::sleep(std::time::Duration::from_millis(500));
            },
        }
    }
}
