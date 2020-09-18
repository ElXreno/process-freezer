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
        
        if procfs::Meminfo::new().unwrap().mem_available.unwrap() >= 4294967296 /* 4GB */ {
            if !sleeping {
                println!("Mem available, unfreezing all processes and sleeping");
                sleeping = true;
            }
            for process in &to_process {
                if process.stat.state != 'S' {
                    if let Err(error) = signal::kill(Pid::from_raw(process.stat.pid), Signal::SIGCONT) {
                        println!("Failed to send SIGCONT for pid {}! Error: {}", process.stat.pid, error);
                    }
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
                if let Some(process) = to_process.first() {
                    if let Err(error) = signal::kill(Pid::from_raw(process.stat.pid), Signal::SIGCONT) {
                        println!("Failed to send SIGCONT for pid {}! Error: {}", process.stat.pid, error);
                    }
                }
                
                std::thread::sleep(std::time::Duration::from_millis(500));
            },
            _ => {
                let max = to_process.iter().max_by(|x, y| x.status().unwrap().vmrss.unwrap().cmp(&y.status().unwrap().vmrss.unwrap())).unwrap();
                
                for process in &to_process {
                    if process.stat.pid != max.stat.pid && process.stat.state != 'T'{
                        println!("Freezing {}", process.stat.pid);
                        if let Err(error) = signal::kill(Pid::from_raw(process.stat.pid), Signal::SIGSTOP) {
                            println!("Failed to send SIGSTOP for pid {}! Error: {}", max.stat.pid, error);
                        }
                    }
                }
                
                if max.stat.state != 'S' {
                    println!("Unfreezing {}", max.stat.pid);
                    if let Err(error) = signal::kill(Pid::from_raw(max.stat.pid), Signal::SIGCONT) {
                        println!("Failed to send SIGCONT for pid {}! Error: {}", max.stat.pid, error);
                    }
                }
                
                std::thread::sleep(std::time::Duration::from_millis(500));
            },
        }
    }
}
