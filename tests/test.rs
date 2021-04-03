#[cfg(test)]

extern crate ruscaml;

use std::fs;
use std::io::{ Write, BufWriter};
use std::process::Command;

#[test]
fn unittest() -> Result<(), Box<dyn std::error::Error>> {
    
    let mut programset = fs::read_to_string("./tests/test.ml").expect("failed to read test.ml.");
    programset = programset.replace(";;\n", ";;");
    let mut programs: Vec<&str> = programset.split(";;").collect();
    programs.pop().unwrap();

    for program in programs {
        let mut f = BufWriter::new(fs::File::create("./tests/onetest.ml").unwrap());
        f.write(program.as_bytes()).unwrap();
        f.write(";;".as_bytes()).unwrap();
        f.flush()?;
        print!("{};; => \n\n", program);
        let output = Command::new("./target/debug/ruscaml")
            .arg("./tests/onetest.ml")
            .output()
            .expect("failed to execute unit test");
        
        let test_stdout = output.stdout;
        let error_stdout = output.stderr;

        println!("{}", std::str::from_utf8(&test_stdout).unwrap());
        if !(std::str::from_utf8(&error_stdout).unwrap() == "") {
            println!("{}", std::str::from_utf8(&error_stdout).unwrap());
            let failed = Command::new("echo")
                .arg("-e")
                .arg(format!("\\e[31m FAILED\\e[m [{}]", program))
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
        println!("##########################################");
    }

    let _ = Command::new("rm")
        .arg("./tests/onetest.ml")
        .output()
        .expect("failed to delete onetest.ml");
    Ok(())
}