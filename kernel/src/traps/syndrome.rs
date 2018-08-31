#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Fault {
    AddressSize,
    Translation,
    AccessFlag,
    Permission,
    Alignment,
    TlbConflict,
    Other(u8)
}

impl From<u32> for Fault {
    fn from(val: u32) -> Fault {
        use self::Fault::*;
        let ifsc = (val & 0b111111) as u8; // Instruction Fault Status Code
        match ifsc {
            0b000000 ... 0b000011 => AddressSize,
            0b000100 ... 0b000111 => Translation,
            0b001001 ... 0b001010 => AccessFlag,
            0b001101 ... 0b001111 => Permission,
            0b100001 => Alignment,
            0b110000 => TlbConflict,
            other => Other(other)
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Syndrome {
    Unknown,
    WfiWfe,
    McrMrc,
    McrrMrrc,
    LdcStc,
    SimdFp,
    Vmrs,
    Mrrc,
    IllegalExecutionState,
    Svc(u16),
    Hvc(u16),
    Smc(u16),
    MsrMrsSystem,
    InstructionAbort {
        kind: Fault,
        level: u8,
    },
    PCAlignmentFault,
    DataAbort {
        kind: Fault,
        level: u8
    },
    SpAlignmentFault,
    TrappedFpu,
    SError,
    Breakpoint,
    Step,
    Watchpoint,
    Brk(u16),
    Other(u32)
}

/// Converts a raw syndrome value (ESR) into a `Syndrome` (ref: D1.10.4).
impl From<u32> for Syndrome {
    fn from(esr: u32) -> Syndrome {
        use self::Syndrome::*;
        match (esr >> 26) {
            0b000000 => Unknown,
            0b000001 => WfiWfe,
            0b000011 | 0b000101 => McrMrc,
            0b000100 => McrrMrrc,
            0b000110 => LdcStc,
            0b000111 => SimdFp,
            0b001000 => Vmrs,
            0b001100 => Mrrc,
            0b001110 => IllegalExecutionState,
            0b010001 | 0b010101 => Svc((esr & 0xFFFF) as u16),
            0b010010 | 0b010110 => Hvc((esr & 0xFFFF) as u16),
            0b010011 | 0b010111 => Smc((esr & 0xFFFF) as u16),
            0b011000 => MsrMrsSystem,
            0b100000 | 0b100001 => InstructionAbort {
                kind: esr.into(),
                level: (esr & 0b11) as u8
            },
            0b100010 => PCAlignmentFault,
            0b100100 | 0b100101 => DataAbort {
                kind: esr.into(),
                level: (esr & 0b11) as u8
            },
            0b100110 => SpAlignmentFault,
            0b101000 | 0b101100 => TrappedFpu,
            0b101111 => SError,
            0b110000 | 0b110001 => Breakpoint,
            0b110010 | 0b110011 => Step,
            0b110100 | 0b110101 => Watchpoint,
            0b111000 | 0b111100 => Brk((esr & 0xFFFF) as u16),
            other => Other(other)
        }
    }
}
