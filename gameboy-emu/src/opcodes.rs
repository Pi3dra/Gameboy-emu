#![allow(dead_code,unused_imports)]
use crate::cpu::{CPU, Registers, Reg8, Reg16, Operand, Op, Binop, Unop};

#[derive(Copy,Clone)]
pub enum InstrPointer {Binop(fn(&mut CPU, Operand, Operand), ), Unop(fn(&mut CPU, Operand))}

#[derive(Copy,Clone)]
struct Instruction{
    handler : InstrPointer, 
    op_args : Op,
    length : u8,
    t_states : u8,
}

impl Instruction{
    fn set_operands(&mut self, dst : Operand, src : Operand) -> Instruction{
        self.src = src;
        self.dst = dst;
        *self
    }

}

fn load_opcode_table(){
    let load_r_r_u8 = Instruction {
        handler: CPU::ld_r_r_u8,
        dst: Operand::Reg8(Reg8::A),
        src: Operand::Reg8(Reg8::A),
        length: 1,
        t_states: 4,
    };

    let load_imm_u8 = Instruction {
        handler: CPU::ld_r_r_u8,
        dst: Operand::Reg8(Reg8::A),
        src: Operand::Reg8(Reg8::A),
        length: 1,
        t_states: 4,
    };

    
    let mut table : [Instruction; 256] = [load_r_r_u8;256];

    // Load  immediate u8
    // TODO: Find a way to load immediates from here? 
    table[0x06] = load_imm_u8.clone().set_operands(Operand::Reg8(Reg8::B), Operand::Imm8);
    table[0x16] = load_imm_u8.clone().set_operands(Operand::Reg8(Reg8::D), Operand::Imm8);
    table[0x26] = load_imm_u8.clone().set_operands(Operand::Reg8(Reg8::H), Operand::Imm8);

    table[0x0E] = load_imm_u8.clone().set_operands(Operand::Reg8(Reg8::D), Operand::Imm8);
    table[0x1E] = load_imm_u8.clone().set_operands(Operand::Reg8(Reg8::E), Operand::Imm8);
    table[0x2E] = load_imm_u8.clone().set_operands(Operand::Reg8(Reg8::L), Operand::Imm8);
    table[0x3E] = load_imm_u8.clone().set_operands(Operand::Reg8(Reg8::A), Operand::Imm8);


    //Load register to register u8
    table[0x40] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::B),Operand::Reg8(Reg8::B));
    table[0x50] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::D),Operand::Reg8(Reg8::B));
    table[0x60] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::H),Operand::Reg8(Reg8::B));

    table[0x41] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::B),Operand::Reg8(Reg8::C));
    table[0x51] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::D),Operand::Reg8(Reg8::C));
    table[0x61] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::H),Operand::Reg8(Reg8::C));
    
    table[0x42] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::B),Operand::Reg8(Reg8::D));
    table[0x52] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::D),Operand::Reg8(Reg8::D));
    table[0x62] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::H),Operand::Reg8(Reg8::D));

    table[0x43] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::B),Operand::Reg8(Reg8::E));
    table[0x53] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::D),Operand::Reg8(Reg8::E));
    table[0x63] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::H),Operand::Reg8(Reg8::E));

    table[0x44] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::B),Operand::Reg8(Reg8::H));
    table[0x54] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::D),Operand::Reg8(Reg8::H));
    table[0x64] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::H),Operand::Reg8(Reg8::H));

    table[0x45] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::B),Operand::Reg8(Reg8::L));
    table[0x55] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::D),Operand::Reg8(Reg8::L));
    table[0x65] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::H),Operand::Reg8(Reg8::L));

    table[0x47] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::B),Operand::Reg8(Reg8::A));
    table[0x57] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::D),Operand::Reg8(Reg8::A));
    table[0x67] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::H),Operand::Reg8(Reg8::A));

    table[0x48] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::C),Operand::Reg8(Reg8::B));
    table[0x58] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::E),Operand::Reg8(Reg8::B));
    table[0x68] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::L),Operand::Reg8(Reg8::B));
    table[0x78] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::A),Operand::Reg8(Reg8::B));

    table[0x49] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::C),Operand::Reg8(Reg8::C));
    table[0x59] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::E),Operand::Reg8(Reg8::C));
    table[0x69] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::L),Operand::Reg8(Reg8::C));
    table[0x79] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::A),Operand::Reg8(Reg8::C));

    table[0x4A] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::C),Operand::Reg8(Reg8::D));
    table[0x5A] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::E),Operand::Reg8(Reg8::D));
    table[0x6A] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::L),Operand::Reg8(Reg8::D));
    table[0x7A] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::A),Operand::Reg8(Reg8::D));

    table[0x4B] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::C),Operand::Reg8(Reg8::E));
    table[0x5B] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::E),Operand::Reg8(Reg8::E));
    table[0x6B] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::L),Operand::Reg8(Reg8::E));
    table[0x7B] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::A),Operand::Reg8(Reg8::E));

    table[0x4C] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::C),Operand::Reg8(Reg8::H));
    table[0x5C] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::E),Operand::Reg8(Reg8::H));
    table[0x6C] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::L),Operand::Reg8(Reg8::H));
    table[0x7C] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::A),Operand::Reg8(Reg8::H));

    table[0x4D] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::C),Operand::Reg8(Reg8::L));
    table[0x5D] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::E),Operand::Reg8(Reg8::L));
    table[0x6D] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::L),Operand::Reg8(Reg8::L));
    table[0x7D] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::A),Operand::Reg8(Reg8::L));

    table[0x4F] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::C),Operand::Reg8(Reg8::A));
    table[0x5F] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::E),Operand::Reg8(Reg8::A));
    table[0x6F] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::L),Operand::Reg8(Reg8::A));
    table[0x7F] = load_r_r_u8.clone().set_operands(Operand::Reg8(Reg8::A),Operand::Reg8(Reg8::A));

    // Increment R8
    let add = Instruction {
        handler: CPU::add_to_register,
        dst: Operand::Reg8(Reg8::A),
        src: Operand::Value8(0),
        length: 1,
        t_states: 4,
    };

    table[0x04] = add.clone().set_operands(Operand::Reg8(Reg8::B), Operand::Value8(1));
    table[0x14] = add.clone().set_operands(Operand::Reg8(Reg8::D), Operand::Value8(1));
    table[0x24] = add.clone().set_operands(Operand::Reg8(Reg8::H), Operand::Value8(1));

    table[0x0C] = add.clone().set_operands(Operand::Reg8(Reg8::C), Operand::Value8(1));
    table[0x1C] = add.clone().set_operands(Operand::Reg8(Reg8::E), Operand::Value8(1));
    table[0x2C] = add.clone().set_operands(Operand::Reg8(Reg8::L), Operand::Value8(1));
    table[0x3C] = add.clone().set_operands(Operand::Reg8(Reg8::A), Operand::Value8(1));
    
    // ADD from register r to A
    table[0x80] = add.clone().set_operands(Operand::Reg8(Reg8::A), Operand::Reg8(Reg8::B)); 
    table[0x81] = add.clone().set_operands(Operand::Reg8(Reg8::A), Operand::Reg8(Reg8::C)); 
    table[0x82] = add.clone().set_operands(Operand::Reg8(Reg8::A), Operand::Reg8(Reg8::D)); 
    table[0x83] = add.clone().set_operands(Operand::Reg8(Reg8::A), Operand::Reg8(Reg8::E)); 
    table[0x84] = add.clone().set_operands(Operand::Reg8(Reg8::A), Operand::Reg8(Reg8::H)); 
    table[0x85] = add.clone().set_operands(Operand::Reg8(Reg8::A), Operand::Reg8(Reg8::L)); 

    table[0x87] = add.clone().set_operands(Operand::Reg8(Reg8::A), Operand::Reg8(Reg8::A)); 

    // ADD from register r to A
    table[0xC6] = add.clone().set_operands(Operand::Reg8(Reg8::A), Operand::Imm8); 

    // Decrement R8
    let sub = Instruction {
        handler: CPU::sub_to_register,
        dst: Operand::Reg8(Reg8::A),
        src: Operand::Value8(0),
        length: 1,
        t_states: 4,
    };

    table[0x05] = sub.clone().set_operands(Operand::Reg8(Reg8::B), Operand::Value8(1));
    table[0x15] = sub.clone().set_operands(Operand::Reg8(Reg8::D), Operand::Value8(1));
    table[0x25] = sub.clone().set_operands(Operand::Reg8(Reg8::H), Operand::Value8(1));

    table[0x0D] = sub.clone().set_operands(Operand::Reg8(Reg8::C), Operand::Value8(1));
    table[0x1D] = sub.clone().set_operands(Operand::Reg8(Reg8::E), Operand::Value8(1));
    table[0x2D] = sub.clone().set_operands(Operand::Reg8(Reg8::L), Operand::Value8(1));
    table[0x3D] = sub.clone().set_operands(Operand::Reg8(Reg8::A), Operand::Value8(1));

    // SUB register r to A
    table[0x90] = sub.clone().set_operands(Operand::Reg8(Reg8::A), Operand::Reg8(Reg8::B)); 
    table[0x91] = sub.clone().set_operands(Operand::Reg8(Reg8::A), Operand::Reg8(Reg8::C)); 
    table[0x92] = sub.clone().set_operands(Operand::Reg8(Reg8::A), Operand::Reg8(Reg8::D)); 
    table[0x93] = sub.clone().set_operands(Operand::Reg8(Reg8::A), Operand::Reg8(Reg8::E)); 
    table[0x94] = sub.clone().set_operands(Operand::Reg8(Reg8::A), Operand::Reg8(Reg8::H)); 
    table[0x95] = sub.clone().set_operands(Operand::Reg8(Reg8::A), Operand::Reg8(Reg8::L)); 

    table[0x97] = sub.clone().set_operands(Operand::Reg8(Reg8::A), Operand::Reg8(Reg8::A)); 

    //SUB Imm8 to A
    table[0xD6] = add.clone().set_operands(Operand::Reg8(Reg8::A), Operand::Imm8); 

    let and = Instruction {
        handler: CPU::and,
        dst: Operand::Reg8(Reg8::A),
        src: Operand::Value8(0),
        length: 1,
        t_states: 4,
    };

    // AND register r to A
    table[0xA0] = and.clone().set_operands(Operand::Reg8(Reg8::A), Operand::Reg8(Reg8::B)); 
    table[0xA1] = and.clone().set_operands(Operand::Reg8(Reg8::A), Operand::Reg8(Reg8::C)); 
    table[0xA2] = and.clone().set_operands(Operand::Reg8(Reg8::A), Operand::Reg8(Reg8::D)); 
    table[0xA3] = and.clone().set_operands(Operand::Reg8(Reg8::A), Operand::Reg8(Reg8::E)); 
    table[0xA4] = and.clone().set_operands(Operand::Reg8(Reg8::A), Operand::Reg8(Reg8::H)); 
    table[0xA5] = and.clone().set_operands(Operand::Reg8(Reg8::A), Operand::Reg8(Reg8::L)); 

    table[0xA7] = and.clone().set_operands(Operand::Reg8(Reg8::A), Operand::Reg8(Reg8::A)); 

    // AND Imm8 to A
    table[0xE6] = and.clone().set_operands(Operand::Reg8(Reg8::A), Operand::Imm8); 
 

    let or = Instruction {
        handler: CPU::or,
        dst: Operand::Reg8(Reg8::A),
        src: Operand::Value8(0),
        length: 1,
        t_states: 4,
    };

    // OR register r to A
    table[0x90] = or.clone().set_operands(Operand::Reg8(Reg8::A), Operand::Reg8(Reg8::B)); 
    table[0x91] = or.clone().set_operands(Operand::Reg8(Reg8::A), Operand::Reg8(Reg8::C)); 
    table[0x92] = or.clone().set_operands(Operand::Reg8(Reg8::A), Operand::Reg8(Reg8::D)); 
    table[0x93] = or.clone().set_operands(Operand::Reg8(Reg8::A), Operand::Reg8(Reg8::E)); 
    table[0x94] = or.clone().set_operands(Operand::Reg8(Reg8::A), Operand::Reg8(Reg8::H)); 
    table[0x95] = or.clone().set_operands(Operand::Reg8(Reg8::A), Operand::Reg8(Reg8::L)); 

    table[0x97] = or.clone().set_operands(Operand::Reg8(Reg8::A), Operand::Reg8(Reg8::A)); 

    // OR Imm8 to A
    table[0xF6] = or.clone().set_operands(Operand::Reg8(Reg8::A), Operand::Imm8); 
 




}

