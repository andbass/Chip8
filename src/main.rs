
mod machine;
mod opcode;

use opcode::Opcode;

fn main() {
    println!("{:?}", Opcode::from_u16(0xC20F));
}
