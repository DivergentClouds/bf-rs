use std::{
    env::args,
    fs::File,
    io::{self, Read, Seek, SeekFrom, Write},
};

#[derive(Debug)]
enum BfError {
    TapeUnderflow,
    #[allow(dead_code)]
    UnmatchedStartBracket(io::Error),
    UnmatchedEndBracket,
    EndOfInput,
    InputFailure,
    #[allow(dead_code)]
    OutputFailure(io::Error),
}

#[derive(Debug)]
enum Error {
    BadArgCount,
    #[allow(dead_code)]
    FileNotFound(io::Error),
    #[allow(dead_code)]
    Interpreter(BfError),
}

fn main() -> Result<(), Error> {
    let mut args_iter = args().skip(1);

    let filename = args_iter.next().ok_or(Error::BadArgCount)?;

    let program = &mut File::open(filename).map_err(Error::FileNotFound)?;
    let program_len = program.metadata().map_err(Error::FileNotFound)?.len();

    interpret(program, program_len).map_err(Error::Interpreter)
}

fn interpret(program: &mut File, program_len: u64) -> Result<(), BfError> {
    let mut tape_index: usize = 0;
    let tape = &mut vec![0u8; 256];

    let bracket_stack: &mut Vec<u64> = &mut Vec::new();

    while program.stream_position().unwrap() < program_len {
        let instruction_buffer = &mut [0u8];
        program.read_exact(instruction_buffer).unwrap();

        match instruction_buffer[0] {
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
                bracket_stack.push(program.stream_position().unwrap());

                if tape[tape_index] == 0 {
                    let mut depth: usize = 1;

                    while depth != 0 {
                        program
                            .read_exact(instruction_buffer)
                            .map_err(BfError::UnmatchedStartBracket)?;

                        match instruction_buffer[0] {
                            b'[' => depth += 1,
                            b']' => depth -= 1,
                            _ => (),
                        }
                    }
                }
            }
            b']' => {
                if tape[tape_index] != 0 {
                    if bracket_stack.is_empty() {
                        return Err(BfError::UnmatchedEndBracket);
                    }
                    _ = program.seek(SeekFrom::Start(*bracket_stack.last().unwrap()))
                } else {
                    _ = bracket_stack.pop();
                }
            }
            b',' => {
                let input_buffer: &mut [u8] = &mut [0];

                io::stdin()
                    .read_exact(input_buffer)
                    .map_err(|e| match e.kind() {
                        io::ErrorKind::UnexpectedEof => BfError::EndOfInput,
                        _ => BfError::InputFailure,
                    })?;

                tape[tape_index] = input_buffer[0];
            }
            b'.' => {
                let output: &[u8] = &[tape[tape_index]];

                io::stdout()
                    .write_all(output)
                    .map_err(BfError::OutputFailure)?;
            }
            _ => (),
        }
    }

    Ok(())
}
