#[cfg(test)]

extern crate ruscaml;

use std::fs;
use std::io::{ Write, BufWriter};
use std::process::Command;

#[test]
fn test() -> Result<(), Box<dyn std::error::Error>> {
    
    let programset = fs::read_to_string("./tests/test.ml").expect("failed to read test.ml.");
    let programs: Vec<&str> = programset.split('\n').collect();

    for program in programs {
        let mut f = BufWriter::new(fs::File::create("./tests/oneline.ml").unwrap());
        f.write(program.as_bytes()).unwrap();
        f.flush()?;
        println!("[ {} ] => ", program);
        let output = Command::new("./target/debug/ruscaml")
            .arg("./tests/oneline.ml")
            .output()
            .expect("failed to execute unit test");
        
        let test_stdout = output.stdout;
        let error_stdout = output.stderr;

        println!("{}", std::str::from_utf8(&test_stdout).unwrap());
        println!("{}", std::str::from_utf8(&error_stdout).unwrap())
    }
    Ok(())
}