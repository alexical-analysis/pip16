# pip16

A [RiSC-16](https://user.eng.umd.edu/~blj/RiSC/) virtual machine written in Rust.

RiSC-16 is a 16-bit educational ISA with 8 registers and 8 instructions: `ADD`, `ADDI`, `NAND`, `LUI`, `SW`, `LW`, `BEQ`, and `JALR`. Programs are loaded directly into the flat 64K word memory space and executed from address 0.

## Usage

```
cargo run
```
