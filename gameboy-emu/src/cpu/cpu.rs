#![allow(dead_code, unused_variables)]

//Try this: https://robertheaton.com/gameboy-doctor/

const ZERO: u8 = 7; //Z
const SUBSTRACTION: u8 = 6; //N
const HALFCARRY: u8 = 5; //H
const CARRY: u8 = 4; //C

use super::{FlagCondition, MemAdress, Operand, Reg8, Reg16};
use crate::cpu::opcodes::InstrPointer;
use std::fmt;

// ================================== CPU =============================

// Making the opcode decoding a child of CPU!
pub struct CPU {
    registers: Registers,
    bus: Memory,
    clock: u64,
    ime: bool,         // Interrupt Master Enable
    ime_pending: bool, // For delayed EI
    halted: bool,
    opcode_table: [InstrPointer; 256],
    cb_table: [InstrPointer; 256],
    //for debug purposes
    executing: (u8, InstrPointer),
}

use FlagCondition::*;
use MemAdress::*;
use Operand::*;
use Reg16::*;
impl CPU {
    pub fn print_state(&mut self) {
        let pc_mem = [
            self.bus.read(self.registers.pc),
            self.bus.read(self.registers.pc.wrapping_add(1)),
            self.bus.read(self.registers.pc.wrapping_add(2)),
            self.bus.read(self.registers.pc.wrapping_add(3)),
        ];

        println!("A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}",
            self.registers.a,
            self.registers.f,
            self.registers.b,
            self.registers.c,
            self.registers.d,
            self.registers.e,
            self.registers.h,
            self.registers.l,
            self.registers.sp,
            self.registers.pc,
            pc_mem[0],
            pc_mem[1],
            pc_mem[2],
            pc_mem[3],
        );
    }
    


    pub fn new(rom: Vec<u8>) -> Self {
        let bus = Memory::new(rom.clone());
        let (opcode_table, cb_table) = CPU::build_table();

        println!{"{:?}",opcode_table[0x05]};

        //Blarg tests supposes that we start after BIOS exec
        let registers = Registers {
            a: 0x01,
            f: 0xB0,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            h: 0x01,
            l: 0x4D,
            sp: 0xFFFE,
            pc: 0x0100,
        };

        let executing = (0, InstrPointer::None);

        CPU {
            registers,
            bus,
            clock: 0,
            ime: false,
            ime_pending: false,
            halted: false,
            opcode_table,
            cb_table,
            executing,
        }
    }

    pub fn step(&mut self) {
        if self.halted {
            if self.interrupt_pending() {
                self.halted = false;
            } else {
                self.clock = self.clock.wrapping_add(4);
            return;
        }
        }

        CPU::print_state(self);
        let pc = self.registers.get_16register(PC);
        //println!();
        let opcode = self.bus.read(pc);

        if opcode == 0xCB {
            let opcode2 = self.bus.read(self.registers.pc.wrapping_add(1));
            self.registers.pc = pc.wrapping_add(2);
            self.execute_from_instr(self.cb_table[opcode2 as usize], opcode2);
        } else {
            self.registers.pc = pc.wrapping_add(1);
            self.execute_from_instr(self.opcode_table[opcode as usize], opcode);
        }
        //println!("{:?}", self.registers)
        self.update_ime();
    }

    pub fn run(&mut self) {
        loop {
            self.step();
            // Optional: safety cutoff to prevent infinite loops
            if self.clock > 50_000_000 {
              println!("❌ Timeout or infinite loop. Test failed or hanging.");
               break;
            }
        }
    }

    fn execute_from_instr(&mut self, instr: InstrPointer, opcode: u8) {
        self.executing = (opcode, instr);
        //println!("\n0x{:02X} {:<20} === pc: 0x{:04X}", opcode, instr, self.registers.pc);
        match instr {
            InstrPointer::Const(func, cycles) => {
                func(self);
                self.clock = self.clock.wrapping_add(cycles as u64);
            }
            InstrPointer::Unop(func, op, cycles) => {
                func(self, op);
                self.clock = self.clock.wrapping_add(cycles as u64);
            }
            InstrPointer::Binop(func, op1, op2, cycles) => {
                func(self, op1, op2);
                self.clock = self.clock.wrapping_add(cycles as u64);
            }
            InstrPointer::None => panic!("Unimplemented opcode"),
        }
    }

    fn check_condition(&mut self, op: Operand) -> bool {
        match op {
            Flag(NZ) => return !self.registers.get_flag(ZERO),
            Flag(Z) => return self.registers.get_flag(ZERO),
            Flag(NC) => return !self.registers.get_flag(CARRY),
            Flag(C) => return self.registers.get_flag(CARRY),
            Flag(None) => true,
            _ => panic!("not a flag condition!"),
        }
    }

    fn get_operand_as_u8(&mut self, op: Operand) -> u8 {
        match op {
            R8(register) => self.registers.get_u8register(register),

            Address(AddrR16(register)) => {
                let address = self.registers.get_16register(register);
                self.bus.read(address)
            }
            Address(HLInc) => {
                let address = self.registers.get_16register(HL);
                self.registers.set_16register(HL, address.wrapping_add(1));
                return self.bus.read(address);
            }
            Address(HLDec) => {
                let address = self.registers.get_16register(HL);
                self.registers.set_16register(HL, address.wrapping_sub(1));
                return self.bus.read(address);
            }
            Imm8 => {
                let address = self.registers.get_16register(PC);
                self.registers.set_16register(PC, address.wrapping_add(1));
                return self.bus.read(address);
            }
            Address(ImmAddr16) => {
                let lsb = self.get_operand_as_u8(Imm8);
                let msb = self.get_operand_as_u8(Imm8);
                let address = CPU::fuse_u8(lsb, msb);
                return self.bus.read(address);
            }

            //For LDH func
            Address(ImmAddr8) => {
                let offset = self.get_operand_as_u8(Imm8);
                let address = CPU::fuse_u8(offset, 0xFF);
                return self.bus.read(address);
            }
            Address(AddrR8(register)) => {
                let offset = self.registers.get_u8register(register);
                let address = 0xFF00u16 + offset as u16;
                return self.bus.read(address);
            }
            Value(n) => n as u8,

            //Address(Imm8) -> Do thhis
            _ => panic!("not a u8 operand for get!"),
        }
    }

    fn set_operand_from_u8(&mut self, op: Operand, value: u8) {
        match op {
            R8(register) => self.registers.set_u8register(register, value),

            Address(AddrR16(register)) => {
                let address = self.registers.get_16register(register);
                self.bus.write(address, value)
            }
            Address(HLInc) => {
                let address = self.registers.get_16register(HL);
                self.bus.write(address, value);
                self.registers.set_16register(HL, address.wrapping_add(1));
            }
            Address(HLDec) => {
                let address = self.registers.get_16register(HL);
                self.bus.write(address, value);
                self.registers.set_16register(HL, address.wrapping_sub(1));
            }
            Address(ImmAddr16) => {
                let lsb = self.get_operand_as_u8(Imm8);
                let msb = self.get_operand_as_u8(Imm8);
                let address = CPU::fuse_u8(lsb, msb);
                self.bus.write(address, value);
            }
            //For LDH func
            Address(ImmAddr8) => {
                let offset = self.get_operand_as_u8(Imm8);
                let address = 0xFF00u16 + offset as u16;
                self.bus.write(address, value)
            }
            Address(AddrR8(register)) => {
                let offset = self.registers.get_u8register(register);
                let address = 0xFF00u16 + offset as u16;
                self.bus.write(address, value)
            }

            _ => panic!("not a u8 operand for set!"),
        }
    }

    fn set_operand_to_u16(&mut self, op: Operand, value: u16) {
        match op {
            R16(register) => return self.registers.set_16register(register, value),
            Address(ImmAddr16) => {

                let lsb = self.get_operand_as_u8(Imm8);
                let msb = self.get_operand_as_u8(Imm8);

                let address = CPU::fuse_u8(lsb, msb);
                let (vlsb, vmsb) = CPU::split_u16(value);
                self.bus.write(address, vlsb);
                self.bus.write(address + 1, vmsb);
            }

            _ => panic!("not a u16 operand for set"),
        }
    }

    fn get_operand_as_u16(&mut self, op: Operand) -> u16 {
        match op {
            R16(register) => return self.registers.get_16register(register),
            Imm16 => {
                let lsb: u8 = self.get_operand_as_u8(Imm8);
                let msb: u8 = self.get_operand_as_u8(Imm8);
                return CPU::fuse_u8(lsb, msb);
            }
            Address(Fixed(value)) => return value,
            Value(value) => return value,
            _ => panic!("not a u16 operand for get! "),
        }
    }

    fn update_flags(&mut self, zero: bool, sub: bool, halfcarry: bool, carry: bool) {
        self.registers.set_flag(ZERO, zero);
        self.registers.set_flag(SUBSTRACTION, sub);
        self.registers.set_flag(HALFCARRY, halfcarry);
        self.registers.set_flag(CARRY, carry);
    }

    fn split_u16(value: u16) -> (u8, u8) {
        let lsb = (value & 0xFF) as u8;
        let msb = (value >> 8) as u8;
        (lsb, msb)
    }

    fn fuse_u8(lsb: u8, msb: u8) -> u16 {
        ((msb as u16) << 8) | (lsb as u16)
    }

    // ============= Loading =============

    //this implements both ldh and ld
    pub(crate) fn ld_u8(&mut self, destination: Operand, source: Operand) {
        let val_to_load: u8 = self.get_operand_as_u8(source);
        self.set_operand_from_u8(destination, val_to_load);
    }

    pub(crate) fn ld_u16(&mut self, destination: Operand, source: Operand) {
        // I think this doesn't properly implement LD [a16] SP
        let val_to_load: u16 = self.get_operand_as_u16(source);
        self.set_operand_to_u16(destination, val_to_load);
    }

    pub(crate) fn ld_u16_e8(&mut self, destination: Operand, _source: Operand) {
        let e8 = self.get_operand_as_u8(Operand::Imm8) as i8;
        let sp = self.get_operand_as_u16(Operand::R16(Reg16::SP));

        let result = sp.wrapping_add_signed(e8 as i16);

        self.set_operand_to_u16(destination, result);

        let h = ((sp & 0xF) + ((e8 as u16) & 0xF)) > 0xF;
        let c = ((sp & 0xFF) + ((e8 as u16) & 0xFF)) > 0xFF;
        self.update_flags(false, false, h, c);
    }

    // ============= Arithmetic =============

    pub(crate) fn add(&mut self, op1: Operand, op2: Operand) {
        let value1: u8 = self.get_operand_as_u8(op1);
        let value2: u8 = self.get_operand_as_u8(op2);
        let (result, overflowed) = value1.overflowing_add(value2);

        //Addition always stores back on a register
        self.set_operand_from_u8(op1, result);

        let half_carry: bool = ((value1 & 0xF) + (value2 & 0xF)) > 0xF;
        self.update_flags(result == 0, false, half_carry, overflowed);
    }

    pub(crate) fn adc(&mut self, op1: Operand, op2: Operand) {
        let a: u8 = self.get_operand_as_u8(op1);
        let n: u8 = self.get_operand_as_u8(op2);
        let carry_in: u8 = self.registers.get_flag(CARRY) as u8;

        let sum16 = a as u16 + n as u16 + carry_in as u16;
        let result = sum16 as u8;
        let half_carry = ((a & 0xF) + (n & 0xF) + carry_in) > 0xF;
        let carry = sum16 > 0xFF;

        self.set_operand_from_u8(op1, result);
        self.update_flags(result == 0, false, half_carry, carry);
    }

    pub(crate) fn sbc(&mut self, op1: Operand, op2: Operand) {
        let a = self.get_operand_as_u8(op1);
        let n = self.get_operand_as_u8(op2);
        let carry_in = self.registers.get_flag(CARRY) as u8;

        let result = a.wrapping_sub(n + carry_in);
        let half_carry = (a & 0xF) < ((n & 0xF) + carry_in);
        let carry = a < n + carry_in;

        self.set_operand_from_u8(op1, result);
        self.update_flags(result == 0, true, half_carry, carry);
    }

    pub(crate) fn inc(&mut self, op1: Operand) {
        let value: u8 = self.get_operand_as_u8(op1);
        let (result, overflowed) = value.overflowing_add(1);

        //Addition always stores back on a register
        self.set_operand_from_u8(op1, result);

        let half_carry: bool = ((value & 0xF) + (1 & 0xF)) > 0xF;
        let carry: bool = self.registers.get_flag(CARRY);
        self.update_flags(result == 0, false, half_carry, carry);
    }

    pub(crate) fn sub(&mut self, op1: Operand, op2: Operand) {
        let value1: u8 = self.get_operand_as_u8(op1);
        let value2: u8 = self.get_operand_as_u8(op2);
        let result = value1.wrapping_sub(value2);

        //Addition always stores back on a register
        self.set_operand_from_u8(op1, result);

        let half_carry: bool = (value1 & 0xF) < (value2 & 0xF);
        let c_flag = value1 < value2; // full-borrow
        self.update_flags(result == 0, true, half_carry, c_flag);
    }

    pub(crate) fn dec(&mut self, op1: Operand) {
        let value: u8 = self.get_operand_as_u8(op1);
        let result = value.wrapping_sub(1);

        self.set_operand_from_u8(op1, result);

        let half_carry: bool = (value & 0xF) == 0;
        let keep_carry: bool = self.registers.get_flag(CARRY);
        self.update_flags(result == 0, true, half_carry, keep_carry);
    }

    pub(crate) fn and(&mut self, source: Operand, op: Operand) {
        let register_value = self.get_operand_as_u8(source);
        let result: u8 = register_value & self.get_operand_as_u8(op);

        self.set_operand_from_u8(source, result);
        self.update_flags(result == 0, false, true, false);
    }

    pub(crate) fn or(&mut self, source: Operand, op: Operand) {
        let register_value = self.get_operand_as_u8(source);
        let result: u8 = register_value | self.get_operand_as_u8(op);

        self.set_operand_from_u8(source, result);
        self.update_flags(result == 0, false, false, false);
    }

    pub(crate) fn xor(&mut self, source: Operand, op: Operand) {
        let register_value = self.get_operand_as_u8(source);
        let result: u8 = register_value ^ self.get_operand_as_u8(op);
        self.set_operand_from_u8(source, result);

        self.update_flags(result == 0, false, false, false);
    }

    pub(crate) fn cp(&mut self, source: Operand, op: Operand) {
        let register_value = self.get_operand_as_u8(source);
        let to_sub: u8 = self.get_operand_as_u8(op);

        let (result, carry) = register_value.overflowing_sub(to_sub);

        let half_carry: bool = (register_value & 0xF) < (to_sub & 0xF);
        self.update_flags(result == 0, true, half_carry, carry);
    }

    // ============= reg16 Arithmetic =============
    pub(crate) fn inc_u16(&mut self, op: Operand) {
        let value = self.get_operand_as_u16(op);
        self.set_operand_to_u16(op, value + 1);
    }

    pub(crate) fn dec_u16(&mut self, op: Operand) {
        let value = self.get_operand_as_u16(op);
        self.set_operand_to_u16(op, value - 1);
    }

    pub(crate) fn add_hl_rr(&mut self, src: Operand) {
        let hl = self.registers.get_16register(Reg16::HL);
        let value = self.get_operand_as_u16(src);

        let (result, carry) = hl.overflowing_add(value);
        let half_carry = ((hl & 0x0FFF) + (value & 0x0FFF)) > 0x0FFF;

        self.registers.set_16register(Reg16::HL, result);
        self.update_flags(self.registers.get_flag(ZERO), false, half_carry, carry);
    }

    pub(crate) fn add_sp_e8(&mut self) {
        let sp = self.registers.get_16register(Reg16::SP);
        let offset = self.get_operand_as_u8(Operand::Imm8) as i8 as i16; // signed immediate

        let result = sp.wrapping_add(offset as u16);

        let half_carry = ((sp & 0xF) + ((offset as u16) & 0xF)) > 0xF;
        let carry = ((sp & 0xFF) + ((offset as u16) & 0xFF)) > 0xFF;

        self.registers.set_16register(Reg16::SP, result);
        self.update_flags(false, false, half_carry, carry);
    }

    // ============= Jumps and Calls =============

    pub(crate) fn jr(&mut self, condition: Operand, op: Operand) {
        // this increments PC
        let offset = self.get_operand_as_u8(op) as i8 as i16;

        if self.check_condition(condition) {
            let pc: u16 = self.registers.get_16register(PC);
            self.registers
                .set_16register(PC, (pc as i16).wrapping_add(offset) as u16);
            self.clock = self.clock.wrapping_add(12 as u64);
        } else {
            self.clock = self.clock.wrapping_add(8 as u64);
        }
    }

    pub(crate) fn jp(&mut self, condition: Operand, address: Operand) {
        let address = self.get_operand_as_u16(address);
        if self.check_condition(condition) {
            self.registers.set_16register(PC, address);
            if matches!(R16(HL), address) {
                self.clock = self.clock.wrapping_add(4);
            } else {
                self.clock = self.clock.wrapping_add(16);
            }
        } else {
            self.clock = self.clock.wrapping_add(12 as u64);
        }
    }

    pub(crate) fn call(&mut self, condition: Operand, address_operand: Operand) {
        let addr = self.get_operand_as_u16(address_operand);
        if self.check_condition(condition) {
            let pc = self.registers.get_16register(PC);
            let mut sp = self.registers.get_16register(SP);

            //Address to jump to 

            //storing PC in stack
            let (lsb, msb) = CPU::split_u16(pc);

            sp = sp.wrapping_sub(1);
            self.bus.write(sp, msb); // LSB
            sp = sp.wrapping_sub(1);
            self.bus.write(sp, lsb); // MSB
            
            //updating registers accordingly
            self.registers.set_16register(SP, sp);
            self.registers.set_16register(PC, addr);

            self.clock = self.clock.wrapping_add(24 as u64);
        } else {
            self.clock = self.clock.wrapping_add(12 as u64);
        }
    }

    pub(crate) fn rst(&mut self, address: Operand) {
        //This func is very similar to call
        let address_value = self.get_operand_as_u16(address);
        let pc = self.registers.get_16register(PC);

        let mut sp = self.registers.get_16register(SP);
        sp = sp.wrapping_sub(1);
        self.bus.write(sp, pc as u8); // LSB
        sp = sp.wrapping_sub(1);
        self.bus.write(sp, (pc >> 8) as u8); // MSB
        self.registers.set_16register(SP, sp);

        self.registers.set_16register(PC, address_value as u16);
    }

    pub(crate) fn ret(&mut self, condition: Operand) {
        if self.check_condition(condition) {
            let mut sp = self.registers.get_16register(SP);
            //Not sure of ordering here, is the stack little or big endian?
            let lsb = self.bus.read(sp);
            sp = sp.wrapping_add(1);
            let msb = self.bus.read(sp);
            sp = sp.wrapping_add(1);

            let address = CPU::fuse_u8(lsb, msb);
            self.registers.set_16register(SP, sp);
            self.registers.set_16register(PC, address);

            if matches!(Operand::Flag(None), condition) {
                self.clock = self.clock.wrapping_add(16 as u64);
                self.clock += 16;
            } else {
                self.clock = self.clock.wrapping_add(20 as u64);
            }
        } else {
            self.clock = self.clock.wrapping_add(8 as u64);
        }
    }

    pub(crate) fn reti(&mut self) {
        self.ret(Flag(None));
        self.ei();
    }

    pub(crate) fn push(&mut self, op: Operand) {
        let value = self.get_operand_as_u16(op);
        let (lsb, msb) = CPU::split_u16(value);

        let mut sp = self.registers.get_16register(SP);

        sp = sp.wrapping_sub(1);
        self.bus.write(sp, msb);
        sp = sp.wrapping_sub(1);
        self.bus.write(sp, lsb);

        self.registers.set_16register(SP, sp);
    }

    pub(crate) fn pop(&mut self, op: Operand) {
        let mut sp = self.registers.get_16register(SP);

        let lsb = self.bus.read(sp);
        sp = sp.wrapping_add(1);
        let msb = self.bus.read(sp);
        sp = sp.wrapping_add(1);


        let value = CPU::fuse_u8(lsb, msb);

        self.set_operand_to_u16(op, value);
        self.registers.set_16register(SP, sp);
    }

    // ============= Interrupts and Misc =============

    pub(crate) fn ei(&mut self) {
        self.ime_pending = true;
    }

    pub(crate) fn di(&mut self) {
        self.ime = false; // Immediately disable interrupts
        self.ime_pending = false; // Make sure no pending enable remains
    }

    pub(crate) fn nop(&mut self) {}

    pub(crate) fn halt(&mut self) {
        self.halted = true;

        /*

        FOR FUTURE ME:
        You need to handle the HALT bug:
        if IME = 0 and an interrupt is pending,
        the PC increments incorrectly.
        while running {
            if cpu.halted {
            // Resume execution only if an interrupt is pending
            if cpu.interrupt_pending() {
                cpu.halted = false;
            } else {
                // CPU is halted, skip instruction fetch/execute
                cpu.clock += 4; // CPU still increments clock slowly
                continue;
            }
        }
        */
    }

    //set carry flag
    pub(crate) fn scf(&mut self) {
        let zero = self.registers.get_flag(ZERO);
        self.update_flags(zero, false, false, true);
    }

    //complement carry flag
    pub(crate) fn ccf(&mut self) {
        let zero = self.registers.get_flag(ZERO);
        let carry = self.registers.get_flag(CARRY);
        self.update_flags(zero, false, false, !carry);
    }

    pub(crate) fn cpl(&mut self) {
        let a = self.registers.get_u8register(Reg8::A);
        let result = !a;

        self.registers.set_u8register(Reg8::A, result);
        self.registers.set_flag(SUBSTRACTION, true);
        self.registers.set_flag(HALFCARRY, true);
    }

    pub(crate) fn daa(&mut self) {
    let mut a = self.registers.a;
    let mut adjust = 0u8;

    if !self.registers.get_flag(SUBSTRACTION) {
        if self.registers.get_flag(HALFCARRY) || (a & 0x0F) > 9 {
            adjust |= 0x06;
        }
        if self.registers.get_flag(CARRY) || a > 0x99 {
            adjust |= 0x60;
            self.registers.set_flag(CARRY, true);
        }
        a = a.wrapping_add(adjust);
    } else {
        if self.registers.get_flag(HALFCARRY) {
            adjust |= 0x06;
        }
        if self.registers.get_flag(CARRY){
            adjust |= 0x60;
        }
        a = a.wrapping_sub(adjust);
    }

    self.registers.set_flag(ZERO, a == 0);
    self.registers.a = a;
    self.registers.set_flag(HALFCARRY,false);
    }

    pub(crate) fn rlca(&mut self) {
        self.rlc(R8(Reg8::A));
        self.clock = self.clock.wrapping_sub(4);
        let keep_carry = self.registers.get_flag(CARRY);
        self.update_flags(false, false, false, keep_carry);
    }
    pub(crate) fn rla(&mut self) {
        self.rl(R8(Reg8::A));
        self.clock = self.clock.wrapping_sub(4);
        let keep_carry = self.registers.get_flag(CARRY);
        self.update_flags(false, false, false, keep_carry);
    }
    pub(crate) fn rrca(&mut self) {
        self.rrc(R8(Reg8::A));
        self.clock = self.clock.wrapping_sub(4);
        let keep_carry = self.registers.get_flag(CARRY);
        self.update_flags(false, false, false, keep_carry);
   }
    pub(crate) fn rra(&mut self) {
        self.rr(R8(Reg8::A));
        self.clock = self.clock.wrapping_sub(4);
        let keep_carry = self.registers.get_flag(CARRY);
        self.update_flags(false, false, false, keep_carry);
  }

    pub(crate) fn stop(&mut self, op: Operand) {
        //println!("STOP");
        //panic!("Is STOP really needed?");
        //This has some weird hardware behavior
        //Chek a lot of documentation if implemented in future!
    }
    // Call this after every instruction fetch/execute
    fn update_ime(&mut self) {
        if self.ime_pending {
            self.ime = true;
            self.ime_pending = false;
        }
    }

    fn interrupt_pending(&mut self) -> bool {
        let ie = self.bus.read(0xFFFF);
        let iflag = self.bus.read(0xFF0F);
        (ie & iflag) != 0
    }
    // ==================== ENDOF NEW AND IMPROVED FUNCS ====================

    // $CB Prefixed

    //rorates r in cirular manner to the left,7bit is copied to 0 and carry
    pub(crate) fn rlc(&mut self, op: Operand) {
        let value = self.get_operand_as_u8(op);
        let msb = value >> 7;
        let result = (value << 1) | msb;

        self.set_operand_from_u8(op, result);
        self.update_flags(result == 0, false, false, msb == 1);
    }

    pub(crate) fn rrc(&mut self, op: Operand) {
        let value = self.get_operand_as_u8(op);
        let lsb = value & 1;
        let result = (value >> 1) | (lsb << 7);

        self.set_operand_from_u8(op, result);
        self.update_flags(result == 0, false, false, lsb == 1);
    }

    // shift to the left, but discard to carry, and use carry to fill
    pub(crate) fn rl(&mut self, op: Operand) {
        let value = self.get_operand_as_u8(op);
        let carry = self.registers.get_flag(CARRY) as u8;
        let msb = value >> 7;
        let result = (value << 1) | carry;

        self.set_operand_from_u8(op, result);
        self.update_flags(result == 0, false, false, msb == 1)
    }

    pub(crate) fn rr(&mut self, op: Operand) {
        let value = self.get_operand_as_u8(op);
        let carry = self.registers.get_flag(CARRY) as u8;
        let lsb = value & 1;
        let result = (value >> 1) | (carry << 7);

        self.set_operand_from_u8(op, result);
        self.update_flags(result == 0, false, false, lsb == 1);
    }

    // same as rl but instead place 0 in the made gap
    pub(crate) fn sla(&mut self, op: Operand) {
        let value = self.get_operand_as_u8(op);
        let msb = value >> 7;
        let result = value << 1;

        self.set_operand_from_u8(op, result);
        self.update_flags(result == 0, false, false, msb == 1)
    }
    pub(crate) fn sra(&mut self, op: Operand) {
        let value = self.get_operand_as_u8(op);
        let msb = value >> 7;
        let lsb = value & 1;

        //not certain about this special case
        let result = if matches!(op, Address(AddrR16(HL))) {
            value >> 1
        } else {
            value >> 1 | (msb << 7)
        };

        self.set_operand_from_u8(op, result);
        self.update_flags(result == 0, false, false, lsb == 1)
    }

    pub(crate) fn swap(&mut self, op: Operand) {
        let value = self.get_operand_as_u8(op);
        let result = (value >> 4) | (value << 4);
        self.set_operand_from_u8(op, result);
        self.update_flags(result == 0, false, false, false);
    }

    pub(crate) fn srl(&mut self, op: Operand) {
        let value = self.get_operand_as_u8(op);
        let lsb = value & 1;
        let result = value >> 1;

        self.set_operand_from_u8(op, result);
        self.update_flags(result == 0, false, false, lsb == 1);
    }

    pub(crate) fn bit(&mut self, bit_idx: Operand, op: Operand) {
        let value = self.get_operand_as_u8(op);
        let bit_index = self.get_operand_as_u8(bit_idx);

        let bit_is_0 = (value >> bit_index) & 1 == 0;
        self.registers.set_flag(ZERO, bit_is_0);
        self.registers.set_flag(SUBSTRACTION, false);
        self.registers.set_flag(HALFCARRY, true);
    }

    pub(crate) fn res(&mut self, bit_idx: Operand, op: Operand) {
        let value = self.get_operand_as_u8(op);
        let bit_index = self.get_operand_as_u8(bit_idx);

        let result = value & !(1 << bit_index);
        self.set_operand_from_u8(op, result);
    } //set nth bit to 0 
    pub(crate) fn set(&mut self, bit_idx: Operand, op: Operand) {
        let value = self.get_operand_as_u8(op);
        let bit_index = self.get_operand_as_u8(bit_idx);

        let result = value | (1 << bit_index);
        self.set_operand_from_u8(op, result);
    }
}

// ================================== REGISTERS =============================

pub struct Registers {
    a: u8,
    f: u8, // Flags register 4: carry 5: half_carry 6: sub 7: zero
    b: u8, // BC 16 bits
    c: u8,
    d: u8, // DE 16 bits
    e: u8,
    h: u8, // HL 16 bits
    l: u8,
    sp: u16, // Stack pointer
    pc: u16,
}
impl fmt::Debug for Registers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "A : 0x{:02X} F : 0x{:08b} \nB : 0x{:02X} C : 0x{:02X} \nD : 0x{:02X} E : 0x{:02X} \nH : 0x{:02X} L : 0x{:02X} \nSP : 0x{:04X} \nPC : 0x{:04X}", self. a, self.f, self.b, self.c, self.d, self.e, self.h, self.l, self.sp, self.pc
        )
    }
}
impl Registers {
    //Think about maybe using #[inline(always)]
    fn set_flag(&mut self, bit_idx: u8, flag: bool) {
        if flag {
            self.f = self.f | ((flag as u8) << bit_idx); //Bitwise or 
        } else {
            self.f = self.f & !(1 << bit_idx);
        }
    }

    fn get_flag(&self, bit_idx: u8) -> bool {
        let mask = 1 << bit_idx;
        self.f & mask != 0
    }

    fn get_16register(&self, register: Reg16) -> u16 {
        fn concat_registers(reg1: u8, reg2: u8) -> u16 {
            ((reg1 as u16) << 8) | (reg2 as u16)
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

    fn set_16register(&mut self, register: Reg16, value: u16) {
        fn set_registers(reg1: &mut u8, reg2: &mut u8, value: u16) {
            let (lsb,msb) = CPU::split_u16(value);

            *reg1 = msb;
            *reg2 = lsb;
        }

        match register {
            Reg16::SP => self.sp = value,
            Reg16::PC => self.pc = value,
            Reg16::AF => {
                let (lsb, msb) = CPU::split_u16(value);
                self.a = msb;
                self.f = lsb & 0xF0;
            }
            Reg16::BC => set_registers(&mut self.b, &mut self.c, value),
            Reg16::DE => set_registers(&mut self.d, &mut self.e, value),
            Reg16::HL => set_registers(&mut self.h, &mut self.l, value),
        }
    }

    fn get_u8register(&self, register: Reg8) -> u8 {
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

    fn set_u8register(&mut self, register: Reg8, value: u8) {
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

// ================================== Memory =============================

struct Memory {
    rom0: [u8; 16_384],
    romn: [u8; 16_384],

    vram: [u8; 8_192],
    ram: [u8; 8_192],

    wram1: [u8; 4_096],
    wram2: [u8; 4_096],
    hram: [u8; 127],

    oam: [u8; 160],

    io: [u8; 128],
    interrupt: [u8; 1],
}

impl Memory {
    pub fn check_serial(&mut self) -> Option<String> {
        // Get the serial registers
        let sb = self.io[0x01]; // 0xFF01: serial data (byte to send)
        let sc = self.io[0x02]; // 0xFF02: serial control (start transfer if 0x81)

        if sc == 0x81 {
            // Blargg just requested a serial transfer
            self.io[0x02] = 0; // emulate completion: clear "transfer in progress"
            return Some((sb as char).to_string()); // return the output character
        }

        Option::None // nothing to print this step
    }

    pub fn new(rom: Vec<u8>) -> Self {
        // Split ROM into 0x0000–0x3FFF (bank 0) and 0x4000–0x7FFF (bank 1)
        let mut rom0 = [0u8; 0x4000];
        let mut romn = [0u8; 0x4000];

        for (i, byte) in rom.iter().enumerate() {
            if i < 0x4000 {
                rom0[i] = *byte;
            } else if i < 0x8000 {
                romn[i - 0x4000] = *byte;
            } else {
                break; // ignore any extra bytes (Game Boy ROM limit per bank)
            }
        }

        Self {
            rom0,
            romn,
            vram: [0; 0x2000],
            ram: [0; 0x2000],
            wram1: [0; 0x1000],
            wram2: [0; 0x1000],
            oam: [0; 0xA0],
            io: [0; 0x80],
            hram: [0; 0x7F],
            interrupt: [0; 1],
        }
    }

    /*
    fn write(&mut self, address: u16, value: u8) {
        let (region, address, _, writable) = self.map(address);

        if writable {
            region[address] = value;
        }
    }*/
    fn write(&mut self, address: u16, value: u8) {
    // Handle serial transfer for Blargg tests
    
    /*
    if address == 0xFF02 && value == 0x81 {
        // Blargg requested a transfer
        let c = self.io[0x01] as char; // Read the byte to send
        print!("{}", c);               // Print immediately
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        self.io[0x02] = 0; // Clear "transfer in progress" flag
        return;            // Do not write to underlying memory, optional
    }*/

    let (region, address, _, writable) = self.map(address);

    if writable {
        region[address] = value;
    }
}


    fn read(&mut self, address: u16) -> u8 {
        if address == 0xFF44 {
            return 0x90 //for some gameboy-doctor tests
        }
        let (region, address, readable, _) = self.map(address);
        
        
        //println!("R: addr: 0x{:04X}, value: 0x{:02X}", address , region[address]);
        if readable {
            return region[address];
        }

        0xFF
    }

    fn map(&mut self, address: u16) -> (&mut [u8], usize, bool, bool) {
        match address {
            0x0000..=0x3FFF => (&mut self.rom0, address as usize, true, false),
            0x4000..=0x7FFF => (&mut self.romn, (address - 0x4000) as usize, true, false),

            0x8000..=0x9FFF => (&mut self.vram, (address - 0x8000) as usize, true, true),
            0xA000..=0xBFFF => (&mut self.ram, (address - 0xA000) as usize, true, true),

            0xC000..=0xCFFF => (&mut self.wram1, (address - 0xC000) as usize, true, true),
            0xD000..=0xDFFF => (&mut self.wram2, (address - 0xD000) as usize, true, true),

            // Echo RAM mirrors C000–DDFF
            0xE000..=0xEFFF => (&mut self.wram1, (address - 0xE000) as usize, true, true),
            0xF000..=0xFDFF => (&mut self.wram2, (address - 0xF000) as usize, true, true),

            0xFE00..=0xFE9F => (&mut self.oam, (address - 0xFE00) as usize, true, true),
            0xFEA0..=0xFEFF => (&mut self.oam, 0, false, false), // not usable; return dummy

            0xFF00..=0xFF7F => (&mut self.io, (address - 0xFF00) as usize, true, true),
            0xFF80..=0xFFFE => (&mut self.hram, (address - 0xFF80) as usize, true, true),

            0xFFFF => (&mut self.interrupt, 0, true, true),

        }
    }
}
