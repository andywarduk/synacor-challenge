extern crate byteorder;

use std::io::{stdin, stdout, Read, Write, Seek, SeekFrom, Error};
use std::fs::File;
use byteorder::{ReadBytesExt, LittleEndian};

struct State {
    debug: bool,
    teleport_patch: bool,
    mem: [u16; 32768],
    regs: [u16; 8],
    stack: Vec<u16>,
    pc: usize,
    last_pc: usize
}

fn main() -> Result<(), Error> {
    let mut state = State {
        debug: false,
        teleport_patch: false,
        mem: [0; 32768],
        regs: [0; 8],
        stack: Vec::new(),
        pc: 0,
        last_pc: 0
    };

    load_prog("challenge.bin", &mut state.mem)?;

    fn get_opcode(state: &mut State) -> u16 {
        let result = state.mem[state.pc];

        state.pc += 1;

        result
    }

    fn get_operand(state: &mut State) -> u16 {
        let result = state.mem[state.pc];

        if state.debug {
            if (result & 0x8000) == 0 { print!(" #{}", result); }
            else { print!(" r{}", result & 0x7fff); }
        }

        state.pc += 1;

        result
    }

    fn get_value(value: u16, state: &mut State) -> u16 {
        if (value & 0x8000) == 0 {
            value
        } else {
            let reg = value & 0x7fff;
            state.regs[reg as usize]
        }
    }

    fn set_value(target: u16, value: u16, state: &mut State) {
        if (target & 0x8000) == 0 {
            panic!("Set target is literal");
        } else {
            state.regs[(target & 0x7fff) as usize] = get_value(value, state);
        }
    }

    loop {
        state.last_pc = state.pc;

        if state.pc == 6027 && state.teleport_patch {
            println!("Patching Ackermann function...");

            state.regs[0] = 6;

            if let Some(retaddr) = state.stack.pop() {
                state.pc = retaddr as usize;
            } else {
                break;
            }

            continue;
        }

        if state.debug { print!("<pc {}: ", state.pc); }

        let opcode = get_opcode(&mut state);

        match opcode {
            0 => {
                if state.debug { print!("halt"); }

                break
            }
            1 => {
                if state.debug { print!("set"); }

                let reg = get_operand(&mut state);
                let reg_lit = get_operand(&mut state);

                set_value(reg, reg_lit, &mut state);
            }
            2 => {
                if state.debug { print!("push"); }

                let val = get_operand(&mut state);

                let push_val = get_value(val, &mut state);

                state.stack.push(push_val);
            }
            3 => {
                if state.debug { print!("pop"); }

                let target = get_operand(&mut state);

                if let Some(val) = state.stack.pop() {
                    set_value(target, val, &mut state);
                } else {
                    panic!("Stack underflow")
                }
            }
            4 => {
                if state.debug { print!("eq"); }

                let target = get_operand(&mut state);
                let val1 = get_operand(&mut state);
                let val2 = get_operand(&mut state);

                if get_value(val1, &mut state) == get_value(val2, &mut state) {
                    set_value(target, 1, &mut state);
                } else {
                    set_value(target, 0, &mut state);
                }
            }
            5 => {
                if state.debug { print!("gt"); }

                let target = get_operand(&mut state);
                let val1 = get_operand(&mut state);
                let val2 = get_operand(&mut state);

                if get_value(val1, &mut state) > get_value(val2, &mut state) {
                    set_value(target, 1, &mut state);
                } else {
                    set_value(target, 0, &mut state);
                }
            }
            6 => {
                if state.debug { print!("jmp"); }

                let new_pc = get_operand(&mut state);

                state.pc = new_pc as usize;
            }
            7 => {
                if state.debug { print!("jt"); }

                let test_reg_lit = get_operand(&mut state);
                let target_pc = get_operand(&mut state);

                if get_value(test_reg_lit, &mut state) != 0 {
                    state.pc = target_pc as usize;
                }
            }
            8 => {
                if state.debug { print!("jf"); }

                let test_reg_lit = get_operand(&mut state);
                let target_pc = get_operand(&mut state);

                if get_value(test_reg_lit, &mut state) == 0 {
                    state.pc = target_pc as usize;
                }
            }
            9 => {
                if state.debug { print!("add"); }

                let target = get_operand(&mut state);
                let val1 = get_operand(&mut state);
                let val2 = get_operand(&mut state);

                let sum = get_value(val1, &mut state) + get_value(val2, &mut state);

                set_value(target, sum % 32768, &mut state);
            }
            10 => {
                if state.debug { print!("mult"); }

                let target = get_operand(&mut state);
                let val1 = get_operand(&mut state);
                let val2 = get_operand(&mut state);

                let product = get_value(val1, &mut state) as u32 * get_value(val2, &mut state) as u32;

                set_value(target, (product % 32768) as u16, &mut state);
            }
            11 => {
                if state.debug { print!("mod"); }

                let target = get_operand(&mut state);
                let val1 = get_operand(&mut state);
                let val2 = get_operand(&mut state);

                let modulus = get_value(val1, &mut state) % get_value(val2, &mut state);

                set_value(target, modulus, &mut state);
            }
            12 => {
                if state.debug { print!("and"); }

                let target = get_operand(&mut state);
                let val1 = get_operand(&mut state);
                let val2 = get_operand(&mut state);

                let result = get_value(val1, &mut state) & get_value(val2, &mut state);

                set_value(target, result, &mut state);
            }
            13 => {
                if state.debug { print!("or"); }

                let target = get_operand(&mut state);
                let val1 = get_operand(&mut state);
                let val2 = get_operand(&mut state);

                let result = get_value(val1, &mut state) | get_value(val2, &mut state);

                set_value(target, result, &mut state);
            }
            14 => {
                if state.debug { print!("not"); }

                let target = get_operand(&mut state);
                let val = get_operand(&mut state);

                let result = (!get_value(val, &mut state)) & 0x7fff;

                set_value(target, result, &mut state);
            }
            15 => {
                if state.debug { print!("rmem"); }

                let target = get_operand(&mut state);
                let source = get_operand(&mut state);

                let result = state.mem[get_value(source, &mut state) as usize];

                set_value(target, result, &mut state);
            }
            16 => {
                if state.debug { print!("wmem"); }

                let target = get_operand(&mut state);
                let source = get_operand(&mut state);

                let result = get_value(source, &mut state);
                state.mem[get_value(target, &mut state) as usize] = result;
            }
            17 => {
                if state.debug { print!("call"); }

                let target = get_operand(&mut state);

                state.stack.push(state.pc as u16);
                state.pc = get_value(target, &mut state) as usize;
            }
            18 => {
                if state.debug { print!("ret"); }

                if let Some(retaddr) = state.stack.pop() {
                    state.pc = retaddr as usize;
                } else {
                    break
                }
            }
            19 => {
                if state.debug { print!("out"); }

                let val = get_operand(&mut state);

                let cu16 = get_value(val, &mut state);

                if let Some(c) = std::char::from_u32(cu16 as u32){
                    print!("{}", c);
                } else {
                    print!("?");
                }
                stdout().flush()?;
            }
            20 => {
                if state.debug { print!("in"); }

                let target = get_operand(&mut state);

                if state.debug { println!("..."); }

                let mut byte: [u8; 1] = [0; 1];

                loop {
                    stdin().read(&mut byte)?;

                    match byte[0] as char {
                        'x' => {
                            let mut string: String = String::new();
                            stdin().read_line(&mut string)?;
                            state.debug = !state.debug;
                            println!("Debug: {}", state.debug);
                        }
                        '!' => {
                            let mut string: String = String::new();
                            stdin().read_line(&mut string)?;
                            state.teleport_patch = true;
                            state.regs[7] = 25734;
                            println!("Teleport patched");
                        }
                        _ => {
                            set_value(target, byte[0] as u16, &mut state);
                            break;
                        }
                    }
                }

                if state.debug { print!("..."); }
            }
            21 => {
                if state.debug { print!("noop"); }
            }
            _ => {
                panic!("Unhandled opcode {}", opcode);
            }
        }

        if state.debug { println!("> {:?}", state.regs); }
    }

    Ok(())
}

fn load_prog(filename: &str, mem: &mut [u16]) -> Result<(), Error> {
    let mut input = File::open(filename)?;

    let len: usize = input.seek(SeekFrom::End(0))? as usize / 2;
    input.seek(SeekFrom::Start(0))?;

    input.read_u16_into::<LittleEndian>(&mut mem[..len])?;

    Ok(())
}
