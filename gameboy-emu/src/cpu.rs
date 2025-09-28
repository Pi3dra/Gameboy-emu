#![allow(dead_code,unused_variables)]

#[derive(Copy,Clone)]
pub enum Binop{
    LD , DEC, ADD, SUB, AND, OR, XOR, ADC, SBC, CP
}

#[derive(Copy,Clone)]
pub enum Unop{
    INC, DEC, PUSH
    
}

#[derive(Copy,Clone)]
pub enum Op {
    Binop(Binop, Operand, Operand),
    Unop(Unop, Operand),
}


#[derive(Clone, Copy)]
pub enum Reg16 {AF, BC, DE, HL, SP, PC}
#[derive(Clone, Copy)]
pub enum Reg8 { A, F, B, C, D, E , H, L}

#[derive(Copy,Clone)]
pub enum Operand {
    Reg8(Reg8),
    Reg16(Reg16),
    Imm8,
    Imm16,
    Value8(u8),
    Value16(u16),
    Nil,
}

impl Operand{
    fn as_reg8(&self) -> Reg8 {
        if let Operand::Reg8(value) = self{
            *value
        } else {
            panic!("Not a Reg8");
        }
    }

    fn as_reg16(&self) -> Reg16 {
        if let Operand::Reg16(value) = self{
            *value
        } else {
            panic!("Not a Reg16");
        }
        
    }

    fn as_value8(&self) -> u8 {
        if let Operand::Value8(value) = self{
            *value
        } else {
            panic!("Not a u8");
        }
        
    }

    fn as_value16(&self) -> u16 {
        if let Operand::Value16(value) = self{
            *value
        } else {
            panic!("Not a u8");
        }
        
    }

} 


pub struct CPU{
    registers : Registers,
    bus : MemoryBus,
    clock : u32
}

impl CPU {
     

    pub fn ld_r_r_u8(&mut self, reg_from : Operand, reg_to : Operand){
        let from = self.registers.get_u8register(reg_from.as_reg8());
        self.registers.set_u8register(reg_to.as_reg8(), from);
    }   

    pub fn ld_imm_u8(&mut self, reg_to : Operand, value : Operand){
        self.registers.set_u8register(reg_to.as_reg8(),value.as_value8())
    }

        pub fn sub_to_register(&mut self, reg_to : Operand , value : Operand){
        match value{
            Operand::Reg8(register) => self.sub_value_reg8(reg_to, self.registers.get_u8register(register), false, true),
            Operand::Value8(value) => self.sub_value_reg8(reg_to, value, true, false),
            _ => panic!("todo"),
        }
    }

    
    fn add_value_reg8(&mut self, register : Reg8, to_add : u8, increment : bool, use_carry : bool){

        let register_value : u8 = self.registers.get_u8register(register);

        let (mut result, mut overflowed) = register_value.overflowing_add(to_add);

        //ADC implementation
        if use_carry {
            let (result2, overflowed2) = result.overflowing_add(self.registers.get_flag(CARRY) as u8);
            result = result2;
            overflowed = overflowed | overflowed2;
        }


        self.registers.set_u8register(register,result);
        self.registers.set_flag(ZERO, result == 0);
        self.registers.set_flag(SUBSTRACTION, false);
        let half_carry = ((register_value & 0xF) + (to_add & 0xF)) > 0xF;
        self.registers.set_flag(HALFCARRY, half_carry);

        // to differentiate decreases or increases
        if !increment { self.registers.set_flag(CARRY, overflowed); }
    }

    fn sub_value_reg8(&mut self, reg_to : Operand , to_sub : u8, decrement : bool,use_carry : bool){

        let register : Reg8 = reg_to.as_reg8();
        let register_value : u8 = self.registers.get_u8register(register);

        let (mut result,mut overflowed) = register_value.overflowing_sub(to_sub);
        
        //SBC implementation
        if use_carry {
            let (result2, overflowed2) = result.overflowing_add(self.registers.get_flag(CARRY) as u8);
            result = result2;
            overflowed = overflowed | overflowed2;
        }


        self.registers.set_u8register(register,result);
        self.registers.set_flag(ZERO, result == 0);
        self.registers.set_flag(SUBSTRACTION, false);
        let half_carry = ((register_value & 0xF) + (to_sub & 0xF)) > 0xF;
        self.registers.set_flag(HALFCARRY, half_carry);

        // to differentiate decreases or increases
        if !decrement { self.registers.set_flag(CARRY, overflowed); }
    }

    pub fn and(&mut self, reg1 : Operand, value: Operand){
        match value {
            Operand::Imm8 => panic!("Implement"),
            Operand::Reg8(value) =>  self.and_register(reg1, self.registers.get_u8register(value)),
            _ => panic!("this should not be happening"),
        }
    }

    pub fn or(&mut self, reg1 : Operand, value: Operand){
        match value {
            Operand::Imm8 => panic!("Implement"),
            Operand::Reg8(value) =>  self.or_register(reg1, self.registers.get_u8register(value)),
            _ => panic!("this should not be happening"),
        }
    }

    pub fn xor(&mut self, reg1 : Operand, value: Operand){
        match value {
            Operand::Imm8 => panic!("Implement"),
            Operand::Reg8(value) =>  self.xor_register(reg1, self.registers.get_u8register(value)),
            _ => panic!("this should not be happening"),
        }
    }

    fn and_register(&mut self, reg1 : Operand, value : u8){
        let register = reg1.as_reg8();
        let result : u8 = self.registers.get_u8register(register) & value;
        self.registers.set_u8register(register, result);

        self.registers.set_flag(ZERO, result == 0);
        self.registers.set_flag(SUBSTRACTION, false);
        self.registers.set_flag(HALFCARRY, true);
        self.registers.set_flag(CARRY, false); 
    }

    fn or_register(&mut self, reg1 : Operand, value : u8){
        let register = reg1.as_reg8();
        let result : u8 = self.registers.get_u8register(register) | value;
        self.registers.set_u8register(register, result);

        self.registers.set_flag(ZERO, result == 0);
        self.registers.set_flag(SUBSTRACTION, false);
        self.registers.set_flag(HALFCARRY, false);
        self.registers.set_flag(CARRY, false); 
    }

    fn xor_register(&mut self, reg1 : Operand, value : u8){
        let register = reg1.as_reg8();
        let result : u8 = self.registers.get_u8register(register) ^ value;
        self.registers.set_u8register(register, result);

        self.registers.set_flag(ZERO, result == 0);
        self.registers.set_flag(SUBSTRACTION, false);
        self.registers.set_flag(HALFCARRY, false);
        self.registers.set_flag(CARRY, false); 
    }
    

    pub fn add_to_register(&mut self, test : Op){
        match test{
            Op::Binop( opname, op1, op2) =>
                match (opname, op1, op2) {
                    (Binop::ADD, Operand::Reg8(register), Operand::Reg8(register2)) => 
                        self.add_value_reg8(register, self.registers.get_u8register(register2), false, true),
                    (Binop::ADD, Operand::Reg8(register), Operand::Imm8) => print!("TODO"),

                    (Binop::ADC, op1, op2) => print!("TODO"),
                    (_,_,_) => panic!("wtf")
                }
            Op::Unop(opname, Operand::Reg8(register)) => 
                self.add_value_reg8(register, 1, false, true),
                
        _ => panic!("This is not an addition!"),
    }
}

}


pub struct MemoryBus {
    memory : [u8; 0xFFFF]
}

const ZERO : u8  = 7; //Z
const SUBSTRACTION : u8 = 6; //N
const HALFCARRY : u8 = 5; //H
const CARRY : u8 = 4; //C

pub struct Registers {
    a : u8,
    f : u8, // Flags register 4: carry 5: half_carry 6: sub 7: zero
    b : u8, // BC 16 bits
    c : u8,
    d : u8, // DE 16 bits
    e : u8,
    h : u8, // HL 16 bits
    l : u8,
    sp : u16, // Stack pointer
    pc : u16, // Program Counter
}

impl Registers {

    fn set_flag(&mut self, bit_idx : u8, flag : bool){
        self.f = self.f | ((flag as u8) << bit_idx) //Bitwise or 
    }

    fn get_flag(& self, bit_idx : u8) -> bool{
        let mask = 1 << bit_idx;
        self.f & mask != 0
    }

    fn get_16register(&self, register : Reg16) -> u16 {
        fn concat_registers(reg1 : u8, reg2: u8) -> u16 {
            ((reg1 as u16) << 8 ) | (reg2 as u16)
        }
        match register {
            Reg16::SP => self.sp,
            Reg16::PC => self.pc,
            Reg16::AF => concat_registers(self.a, self.f),
            Reg16::BC => concat_registers(self.b, self.c),
            Reg16::DE => concat_registers(self.d, self.e),
            Reg16::HL => concat_registers(self.h, self.l),
        }
    } 

    fn set_16register(&mut self, register : Reg16, value : u16){

        fn set_registers(reg1 : &mut u8, reg2: &mut u8, value : u16){
            let high: u8 = (value >> 8) as u8;
            let low: u8 = (value & 0xFF) as u8;

            *reg1 = high;
            *reg2 = low;
        }

        match register {
            Reg16::SP => self.sp = value,
            Reg16::PC => self.pc = value,
            Reg16::AF => set_registers(&mut self.a, &mut self.f, value),
            Reg16::BC => set_registers(&mut self.b, &mut self.c, value),
            Reg16::DE => set_registers(&mut self.d, &mut self.e, value),
            Reg16::HL => set_registers(&mut self.h, &mut self.l, value),
        }
    }

    fn get_u8register(&self , register : Reg8) -> u8 {
        match register {
            Reg8::A => self.a,
            Reg8::F => self.f,
            Reg8::B => self.b,
            Reg8::C => self.c,
            Reg8::D => self.d,
            Reg8::L => self.l,
            Reg8::H => self.h,
            Reg8::E => self.e,
        }
    }

    fn set_u8register(&mut self , register : Reg8 , value : u8){
        match register {
            Reg8::A => self.a = value,
            Reg8::F => self.f = value,
            Reg8::B => self.b = value,
            Reg8::C => self.c = value,
            Reg8::D => self.d = value,
            Reg8::L => self.l = value,
            Reg8::H => self.h = value,
            Reg8::E => self.e = value,
        }
    }    
}
