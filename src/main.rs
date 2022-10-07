use anyhow::{bail, Error};
use clap::Parser;
use std::{
    fs::File,
    io::{stdin, Read},
    path::PathBuf,
};

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    long_about = None
)]
struct Args {
    file: PathBuf,
}

#[derive(Clone)]
enum OpCode {
    PlusP,
    MinusP,
    Plus,
    Minus,
    Write,
    Read,
    LoopBegin,
    LoopEnd,
}
impl OpCode {
    fn from(source: String) -> Vec<OpCode> {
        let mut operations = Vec::new();
        for symbol in source.chars() {
            let op = match symbol {
                '>' => Some(OpCode::PlusP),
                '<' => Some(OpCode::MinusP),
                '+' => Some(OpCode::Plus),
                '-' => Some(OpCode::Minus),
                '.' => Some(OpCode::Write),
                ',' => Some(OpCode::Read),
                '[' => Some(OpCode::LoopBegin),
                ']' => Some(OpCode::LoopEnd),
                _ => None,
            };
            match op {
                Some(op) => operations.push(op),
                None => (),
            }
        }

        operations
    }
}

#[derive(Clone)]
enum Instruction {
    PlusP,
    MinusP,
    Plus,
    Minus,
    Write,
    Read,
    Loop(Vec<Instruction>),
}
impl Instruction {
    fn from(opcodes: Vec<OpCode>) -> Result<Vec<Instruction>, Error> {
        let mut program: Vec<Instruction> = Vec::new();
        let mut loop_stack = 0;
        let mut loop_start = 0;

        for (i, op) in opcodes.iter().enumerate() {
            if loop_stack == 0 {
                let instr = match op {
                    OpCode::PlusP => Some(Instruction::PlusP),
                    OpCode::MinusP => Some(Instruction::MinusP),
                    OpCode::Plus => Some(Instruction::Plus),
                    OpCode::Minus => Some(Instruction::Minus),
                    OpCode::Write => Some(Instruction::Write),
                    OpCode::Read => Some(Instruction::Read),
                    OpCode::LoopBegin => {
                        loop_start = i;
                        loop_stack += 1;
                        None
                    }
                    OpCode::LoopEnd => bail!("Loop ending at #{} has no beginning", i),
                };

                match instr {
                    Some(instr) => program.push(instr),
                    None => (),
                }
            } else {
                match op {
                    OpCode::LoopBegin => {
                        loop_stack += 1;
                    }
                    OpCode::LoopEnd => {
                        loop_stack -= 1;

                        if loop_stack == 0 {
                            program.push(Instruction::Loop(Instruction::from(
                                opcodes[loop_start + 1..i].to_vec(),
                            )?));
                        }
                    }
                    _ => (),
                }
            }
        }

        if loop_stack != 0 {
            bail!(
                "Loop that starts at #{} has no matching ending!",
                loop_start
            );
        }

        Ok(program)
    }
}

fn run(
    instructions: &Vec<Instruction>,
    tape: &mut Vec<u8>,
    data_pointer: &mut usize,
) -> Result<(), Error> {
    let mut inp_c = 0;
    for instr in instructions {
        match instr {
            Instruction::PlusP => *data_pointer += 1,
            Instruction::MinusP => *data_pointer -= 1,
            Instruction::Plus => tape[*data_pointer] += 1,
            Instruction::Minus => tape[*data_pointer] -= 1,
            Instruction::Write => print!("{}", tape[*data_pointer] as char),
            Instruction::Read => {
                let mut input: [u8; 1] = [0; 1];
                println!("Input {}:", inp_c);
                stdin().read_exact(&mut input)?;
                inp_c += 1;
                tape[*data_pointer] = input[0];
            }
            Instruction::Loop(nested_instructions) => {
                while tape[*data_pointer] != 0 {
                    run(&nested_instructions, tape, data_pointer)?
                }
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), Error> {
    let args = Args::parse();
    let file_name = args.file;
    if !file_name.exists() {
        bail!("Input file does not exist: {}", file_name.to_string_lossy());
    }

    let mut file = File::open(file_name).expect("Input file could not be read");
    let mut source = String::new();
    file.read_to_string(&mut source)
        .expect("Failed to read input file");

    let mut tape: Vec<u8> = vec![0; 1024];
    let mut data_pointer = 512;

    run(
        &Instruction::from(OpCode::from(source))?,
        &mut tape,
        &mut data_pointer,
    )?;

    Ok(())
}
