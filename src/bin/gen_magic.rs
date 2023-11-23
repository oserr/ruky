use ruky::magics::{compute_bmagics, compute_rmagics, Magics};
use std::thread::spawn;

fn main() {
    println!("Beginning to compute magics...");

    let t = spawn(|| compute_bmagics());
    let r = compute_rmagics();

    let br = t.join();
    if br.is_err() {
        exit_with_err("Unable to join thread for bmagics.");
    }

    match (br.unwrap(), r) {
        (Ok(ref bmagic), Ok(ref rmagic)) => process_magics(bmagic, rmagic),
        (Ok(_), Err(_)) => exit_with_err("Error computing rook magics."),
        (Err(_), Ok(_)) => exit_with_err("Error computing bishop magics."),
        _ => exit_with_err("Error computing rook and bishop magics."),
    }
}

fn process_magics(bmagics: &impl Magics, rmagics: &impl Magics) {
    print_info(bmagics, false);
    print_info(rmagics, true);
}

fn print_info(magics: &impl Magics, for_rook: bool) {
    println!(
        "{} magics are ...",
        if for_rook { "Rook" } else { "Bishop" }
    );
    for (i, m) in magics.as_ref().iter().enumerate() {
        println!(
            "\tsquare {i}: magic={} mask={:?} rshift={} num_attacks={}",
            m.magic,
            m.mask,
            m.rshift,
            m.attacks.len()
        );
    }
}

fn exit_with_err(msg: &str) {
    eprintln!("{}", msg);
    std::process::exit(1);
}
