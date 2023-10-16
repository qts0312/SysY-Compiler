mod ast;
mod tools;
mod mem;
mod ir;
mod asm;

use lalrpop_util::lalrpop_mod;
use mem::generate_mem;
use ir::generate_ir;
use asm::generate_asm;
use std::env::args;
use std::fs::read_to_string;
use std::io::Result;
use std::time::SystemTime;

lalrpop_mod!(sysy);

fn main() -> Result<()> {
    let mut args = args();
    args.next();
    let mode = args.next().unwrap();
    let input = args.next().unwrap();
    args.next();
    let output = args.next().unwrap();

    let input = read_to_string(input)?;
    let ast = sysy::CompUnitParser::new().parse(&input).unwrap();

    println!("start mem: {}", SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs());
    let (program, mut info) = generate_mem(&ast);
    println!("end mem: {}", SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs());

    if mode == "-koopa" {
        println!("start ir: {}", SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs());
        generate_ir(&program, &output, &info);
        println!("end ir: {}", SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs());
    }
    else {
        println!("start asm: {}", SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs());
        generate_asm(&program, &mut info, &output);
        println!("end asm: {}", SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs());
    }

    Ok(())
}
