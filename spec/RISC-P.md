# The RISC-P ISA

This ISA is an alternative ISA based on the [RiSC-16](https://user.eng.umd.edu/~blj/RiSC/) ISA.
It supports more instructions as well as additional registers while still keeping every instruction
to 16-bits.

## The Registers

The CPU supports 16 registers. 
Register 0 will always contain the value zero, which means read to this register will always return 0.
This is a constraint enforced by the CPU.

Register 13 is used as the frame pointer.
Register 14 is used as the stack pointer.
Register 15 is used as the return address for the JALR instruction if the return address needs to be saved.
Unlike with Register 0 the behavior of Register 13,14 and 15 are not enforced by CPU.
Rather these are program conventions.

The CPU also has a hiddent intterupts register that it uses to save and restore the program counter 
when processing exceptions. This register can not be accessed by user code.

## The Instructions

There are 3 general instruction formats used by the ISA.
Every instruction format uses the top nibble for the opcode (0 - 15).
RRR is the triple register format and is laid out as follows:

| F | E | D | C | B | A | 9 | 8 | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
| ------------- | ------------- | ------------- | ------------- |
| Opcode        | Reg A         | Reg B         | Reg C         |

RRI is the double register format and is laid out as follows:

| F | E | D | C | B | A | 9 | 8 | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
| ------------- | ------------- | ------------- | ------------- |
| Opcode        | Reg A         | Reg B         | Immediate     |

RI is the single register format and is laid out as follows:

| F | E | D | C | B | A | 9 | 8 | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
| ------------- | ------------- | ------------- | ------------- |
| Opcode        | Reg A         | Immediate                     |

### Instructions

| Opcode | Instruction | Format | Full Name            |
| ------ | ----------- | ------ | -------------------- |
| 0b0000 | ADD         | RRR    | Add Registers        |
| 0b0001 | SUB         | RRR    | Sub Registers        |
| 0b0010 | MUL         | RRR    | Mul Registers        |
| 0b0011 | XOR         | RRR    | Xor Registers        |
| 0b0100 | NAND        | RRR    | Nand Registers       |
| 0b0101 | SHL         | RRI    | Shift Left           |
| 0b0110 | SHR         | RRI    | Shift Right          |
| 0b0111 | LUI         | RI     | Load Upper Immediate |
| 0b1000 | LLI         | RI     | Load Lower Immediate  |
| 0b1001 | SW          | RRI    | Store Word           |
| 0b1010 | LW          | RRI    | Load Word            |
| 0b1011 | JALR        | RRI    | Jump and Link        |
| 0b1100 | BEQ         | RRR    | Branch Equal         |
| 0b1101 | BNE         | RRR    | Branch Not Equal     |
| 0b1110 | BLT         | RRR    | Branch Less Than     |
| 0b1111 |             |        | Reserved             |

**ADD** - Adds the contents of Reg C to Reg B and stores the result in Reg A
**SUB** - Subtracts the contents of Reg C from Reg B and stores the result in Reg A
**MUL** - Multiplies the contents of Reg C with Reg B and stores the result in Reg A
**XOR** - XOR the contents of Reg B with Reg C and stores the result in Reg A
**NAND** - Nand the contents of Reg B to Reg C and stores the result in Reg A
**SHL** - Shift the contents of Reg B Immediate bits to the left and stores the result in Reg A
**SHR** - Shift the contents of Reg B Immediate bits to the right and stores the result in Reg A
**LUI** - Shift the Immediate value 8-bits to the left and store the result in Reg A
**LLI** - OR the Immediate into the lower 8 bits of Reg A
**SW** - Store the value in Reg A into the memory address calculated by adding Reg B and the Immediate
**LW** - Load the value in the memory address calculated by adding Reg B and the Immediate into Reg A
**JALR** - Saves the program counter + 1 into Reg A and then branches to the address in Reg B
**BEQ** - If Reg B and Reg C are equal, jump to the address in Reg A
**BNE** - If Reg B and Reg C not equal, jump to the address in Reg A
**BLT** - If Reg B is less than Reg C, jump to the address in Reg A

The final instruction is left as reserved for now. The following options are under consideration.
**SCALL** - System Call instruction with RI format. The Immediate becomes a system call number.
**OR** - Compliments XOR and NAND and shows up a lot in low level code.

## Pseudo Instructions

There are currently a couple pseudo instructions that repurpose otherwise useless instructions.

**NOOP** - The ADD(0,0,0) instruction is treated as a no-op since Register 0 is always 0.
**RETL** - The JALR(0,0,>0) instruction with a non-zero immediate value is treated as a return from 
  interrupt instruction. It automatically returns the program counter to continue normal work using
  the hidden interrupts register.

There are other essentially useless instructions that could also be repurposed but have not been yet.

SHL(\*,\*,0) - Reserved
SHR(\*,\*,0) - Reserved
