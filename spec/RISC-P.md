# The RISC-P ISA

This ISA is an alternative ISA based on the [RiSC-16](https://user.eng.umd.edu/~blj/RiSC/) ISA.
It supports more instructions as well as additional registers while still keeping every instruction
to 16-bits.

## The Instructions

There are 3 general instruction formats used by the ISA.
Every instruction format uses the top nibble for the opcode (0 - 16).
RRR is the tirple register formaat and is layed out as follows:

| F | E | D | C | B | A | 9 | 8 | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
| ------------- | ------------- | ------------- | ------------- |
| Opcode        | Reg A         | Reg B         | Reg C         |

RRI is the doulbe register format and is layed out as follows:

| F | E | D | C | B | A | 9 | 8 | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
| ------------- | ------------- | ------------- | ------------- |
| Opcode        | Reg A         | Reg B         | Immediate     |

RI is the single register format and is layed out as follows:

| F | E | D | C | B | A | 9 | 8 | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
| ------------- | ------------- | ------------- | ------------- |
| Opcode        | Reg A         | Immediate                     |

### Instructions

| Opcode | Instruction | Format | Full Name            |
| ------ | ----------- | ------ | -------------------- |
| 0b0000 | ADD         | RRR    | Add Registers        |
| 0b0001 | ADDI        | RRI    | Add Immediate        |
| 0b0010 | ADDU        | RRI    | Add Upper Immediate  |
| 0b0011 | NAND        | RRR    | Nand Registers       |
| 0b0100 | LUI         | RI     | Load Upper Immediate |
| 0b0100 | XLI         | RI     | XOR Lower Immediate  |
| 0b0101 | SW          | RRI    | Store Word           |
| 0b0110 | LW          | RRI    | Load Word            |
| 0b0111 | BEQ         | RRR    | Register Jump        |
| 0b1000 | JALR        | RRI    | Jump and Link        |
| 0b1001 |             |        |                      |
| 0b1010 |             |        |                      |
| 0b1011 |             |        |                      |
| 0b1100 |             |        |                      |
| 0b1101 |             |        |                      |
| 0b1110 |             |        |                      |
| 0b1111 |             |        |                      |
