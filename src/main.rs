use std::{
    env::args,
    fs::read,
    io::{self, Read, Write},
};

#[derive(Debug)]
enum BfError {
    BadArgCount,
    FileNotFound,
    TapeUnderflow,
    UnmatchedStartBracket,
    UnmatchedEndBracket,
    EndOfInput,
    InputFailure,
    OutputFailure,
}

fn main() -> Result<(), BfError> {
    let mut args_iter = args().skip(1);

    // we already checked arg count so this should be safe
    let filename = match args_iter.next() {
        Some(s) => s,
        None => return Err(BfError::BadArgCount),
    };

    let program = match read(filename) {
        Ok(f) => f,
        Err(_) => return Err(BfError::FileNotFound),
    };

    match interpret(&program) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

fn interpret(program: &[u8]) -> Result<(), BfError> {
    let mut program_counter: usize = 0;
    let mut tape_index: usize = 0;
    let tape: &mut Vec<u8> = &mut vec![0; 256];

    let bracket_stack: &mut Vec<usize> = &mut Vec::new();

    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    while program_counter < program.len() {
        match program[program_counter] {
            b'+' => tape[tape_index] = tape[tape_index].wrapping_add(1),
            b'-' => tape[tape_index] = tape[tape_index].wrapping_sub(1),
            b'>' => {
                tape_index += 1;
                if tape_index == tape.len() {
                    tape.extend([0; 256]);
                }
            }
            b'<' => {
                if tape_index == 0 {
                    return Err(BfError::TapeUnderflow);
                }
                tape_index -= 1;
            }
            b'[' => {
                bracket_stack.push(program_counter);

                if tape[tape_index] == 0 {
                    let mut depth: usize = 1;

                    while depth != 0 {
                        program_counter += 1;
                        match program[program_counter] {
                            b'[' => depth += 1,
                            b']' => depth -= 1,
                            _ => (),
                        }
                    }
                }
            }
            b']' => {
                if tape[tape_index] != 0 {
                    program_counter = match bracket_stack.last() {
                        Some(s) => *s,
                        None => return Err(BfError::UnmatchedEndBracket),
                    }
                } else {
                    _ = bracket_stack.pop();
                }
            }
            b',' => {
                let input: &mut [u8] = &mut [0];
                tape[tape_index] = match stdin.read_exact(input) {
                    Ok(_) => input[0],
                    Err(e) => match e.kind() {
                        io::ErrorKind::UnexpectedEof => return Err(BfError::EndOfInput),
                        _ => return Err(BfError::InputFailure),
                    },
                };
            }
            b'.' => {
                let output: &mut [u8] = &mut [tape[tape_index]];
                match stdout.write_all(output) {
                    Ok(_) => (),
                    Err(_) => return Err(BfError::OutputFailure),
                }
            }
            _ => (),
        }

        program_counter += 1;
    }

    if bracket_stack.is_empty() {
        Ok(())
    } else {
        Err(BfError::UnmatchedStartBracket)
    }
}
