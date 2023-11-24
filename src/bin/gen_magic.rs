use ruky::magics::{compute_bmagics, compute_rmagics, Magics};
use std::env;
use std::fs::File;
use std::io::Write;
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

    if let Some(fname) = env::args_os().nth(1) {
        match File::create(&fname) {
            Err(_) => exit_with_err("Unable to open file for writing."),
            Ok(mut file) => {
                fwrite(
                    &mut file,
                    "BMAGICS",
                    bmagics.as_ref().iter().map(|m| m.magic),
                );
                fwrite(
                    &mut file,
                    "RMAGICS",
                    rmagics.as_ref().iter().map(|m| m.magic),
                );
            }
        }
    }
}

fn fwrite(ofile: &mut File, name: &str, magics: impl Iterator<Item = u64>) {
    if let Err(_) = write!(ofile, "const {}: [u64: 64] = [\n", name) {
        exit_with_err(format!("Unable to write magics for {}", name).as_ref());
    }

    for m in magics {
        if let Err(_) = write!(ofile, "{:#x},\n", m) {
            exit_with_err(format!("Unable to write magics for {}", name).as_ref());
        }
    }

    if let Err(_) = write!(ofile, "];\n") {
        exit_with_err(format!("Unable to write magics for {}", name).as_ref());
    }
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
