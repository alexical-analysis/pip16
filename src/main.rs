mod vm;

use vm::VM;

use crate::vm::{EncInst, Word};

fn main() {
    let mut vm = VM::new();

    vm.load(&[
        EncInst::new_lw(1, 0, 7).into(),   // 0: r1 = memory[0+7] = count (5)
        EncInst::new_lw(2, 1, 3).into(),   // 1: r2 = memory[r1+2] = memory[8] = -1 (neg1)
        EncInst::new_add(1, 1, 2).into(),  // 2 (start): r1 = r1 + r2 (decrement)
        EncInst::new_beq(0, 1, 1).into(),  // 3: if r1==0, jump to done (addr 5)
        EncInst::new_beq(0, 0, -3).into(), // 4: jump to start (addr 2)
        EncInst::new_addi(7, 0, 6).into(), // 5: r7 = r0 + 6 (i.e. 6)
        EncInst::new_jalr(0, 7).into(),    // 6: halt (just loop forever)
        Word::from(5i16),                  // 7: (count): .fill 5
        Word::from(-1i16),                 // 8: (neg1): .fill -1
    ]);

    vm.step();
    print!("(1)\n{}", vm);

    vm.step();
    print!("(2)\n{}", vm);

    vm.step();
    print!("(3)\n{}", vm);

    println!("\nrunning vm...\n");

    for _ in 0..50 {
        vm.step();
    }

    print!("{}", vm);
}
