use ruky::magics::{compute_bmagics, Magics};

fn main() {
    println!("Beginning to compute magics.");
    let magics = compute_bmagics().expect("Unable to compute magics.");
    println!("Computed magics successfully.!");
    for i in 0..64 {
        let magic = magics.get(i).expect("Unable to find magic.");
        println!(
            "Magic for square {i} with magic={:?}, mask={:?} rshift={:?}",
            magic.magic, magic.mask, magic.rshift
        );
    }
}
