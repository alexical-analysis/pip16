# The RiSC-16 ISA

This ISA is take from [RiSC-16](https://user.eng.umd.edu/~blj/RiSC/) and is described as a "Ridiculously 
Simple Computer".
It was designed as teaching tool and the instruction set only has 8-opcodes total.
This makes implemenation easy and execution fast, at the cost of an extreamly limited set of regster actions

## The Instructions

### Instruction Formats

There are 3 general instruction formats used by the ISA.
Every instruction format uses the top 3 bits for the opcode (0 -7).
RRR is the triple register format and is layed out as follows:

| F | E | D | C | B | A | 9 | 8 | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
| --------- | --------- | --------- | ------------- | --------- |
| Opcode    | Reg A     | Reg B     | Reserved      | Reg C     |

RRI is a double register format and is layed out as follows:

| F | E | D | C | B | A | 9 | 8 | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
| --------- | --------- | --------- | ------------- | --------- |
| Opcode    | Reg A     | Reg B     | Signed Immediate Value    |

RI is a single register format and is layed out as follows:

| F | E | D | C | B | A | 9 | 8 | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
| --------- | --------- | --------- | ------------- | --------- |
| Opcode    | Reg A     | Unsigned Immediate Value              |

### Instructions

| Opcode | Instruction | Format | Full Name            |
| ------ | ----------- | ------ | -------------------- |
| 0b000  | ADD         | RRR    | Add Registers        |
| 0b001  | ADDI        | RRI    | Add Immediate        |
| 0b010  | NAND        | RRR    | Nand Registers       |
| 0b011  | LUI         | RI     | Load Upper Immediate |
| 0b100  | SW          | RRI    | Store Word           |
| 0b101  | LW          | RRI    | Load Word            |
| 0b110  | BEQ         | RRI    | Branch If Equal      |
| 0b111  | JALR        | RRI    | Jump and Link        |
