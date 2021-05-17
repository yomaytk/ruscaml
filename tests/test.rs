#[cfg(test)]

extern crate ruscaml;

use std::fs;
use std::io::{ Write, BufWriter};
use std::process::Command;

#[test]
fn unittest() -> Result<(), Box<dyn std::error::Error>> {
    
    let mut programset = fs::read_to_string("./tests/test.ml").expect("failed to read test.ml.");
    programset = programset.replace(":\n", ":");
    let mut programs: Vec<&str> = programset.split(":").collect();
    programs.pop().unwrap();

    for program in programs {
        let mut f = BufWriter::new(fs::File::create("./tests/onetest.ml").unwrap());
        f.write(program.as_bytes()).unwrap();
        f.flush()?;
        print!("{}\n", program);
        let output = Command::new("./target/debug/ruscaml")
            .args(&["./tests/onetest.ml"])
            .output()
            .expect("failed to execute unit test");
        
        let test_stdout = output.stdout;
        let error_stdout = output.stderr;

        if !(std::str::from_utf8(&error_stdout).unwrap() == "") {
            println!("{}", std::str::from_utf8(&error_stdout).unwrap());
            let failed = Command::new("echo")
                .args(&["-e", &format!("\\e[31m FAILED COMPILE\\e[m [{}]", program)])
                .output()
                .expect("");
            let failed_stdout = failed.stdout;
            println!("{}", std::str::from_utf8(&failed_stdout).unwrap());
            let _ = Command::new("rm")
                .arg("./tests/onetest.ml")
                .output()
                .expect("failed to delete onetest.ml");
            std::process::exit(1);
        }

        let mut f2 = BufWriter::new(fs::File::create("a.s").unwrap());
        f2.write(&test_stdout).unwrap();
        f2.flush()?;

        let _ = Command::new("aarch64-linux-gnu-gcc")
            .args(&["-c", "src/exe.c"])
            .output()
            .expect("this command should always sucess");

        let arm64assemble = Command::new("aarch64-linux-gnu-gcc")
            .args(&["exe.o", "a.s", "-o", "a"])
            .output()
            .expect("");
        
        if !arm64assemble.status.success() {
            let _ = Command::new("echo")
                .arg("-e")
                .arg(format!("\\e[31m FAILED ASSEMBLE\\e[m [{}]", program))
                .output()
                .expect("");
            
            let failed_message = arm64assemble.stderr;
            println!("assemble error: {}", std::str::from_utf8(&failed_message).unwrap());
            std::process::exit(1);
        }

        let qemu_exe = Command::new("qemu-aarch64")
            .args(&["-L", "/usr/aarch64-linux-gnu", "a"])
            .output()
            .expect("");        
        
        if !qemu_exe.status.success() {
            let _ = Command::new("echo")
                .arg("-e")
                .arg(format!("\\e[31m FAILED EXECUTION\\e[m [{}]", program))
                .output()
                .expect("");
            std::process::exit(1);
        }
        println!("OK");
    }

        let _ = Command::new("rm")
        .args(&["./tests/onetest.ml", "a", "exe.o"])
        .output()
        .expect("failed to delete onetest.ml");
        
    Ok(())
}