pub mod cpu;
pub mod opcodes;

#[derive(Clone, Copy, Debug)]
pub(crate) enum Reg16 {
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Reg8 {
    A,
    F,
    B,
    C,
    D,
    E,
    H,
    L,
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum MemAdress {
    //memory is u8!
    HLInc,          //[HL+]
    HLDec,          //[HL-]
    AddrR8(Reg8),   //[A]
    AddrR16(Reg16), //[SP]
    ImmAddr8,       //[a8]
    ImmAddr16,      //[a16]
    Fixed(u16),     // $18
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum FlagCondition {
    NZ,   // Not Zero
    Z,    // Zero
    NC,   // Not Carry
    C,    // Carry
    None, // Unconditional (for JR e, CALL nn, etc.)
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum Operand {
    R8(Reg8),
    R16(Reg16),
    Address(MemAdress),
    Flag(FlagCondition),
    Imm8,
    Imm16,
    Value(u16), //Special for rst function
}

pub use cpu::CPU;
