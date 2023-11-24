use ruky::magics::{compute_bmagics, compute_rmagics, from_bmagics, from_rmagics, Magics};
use std::thread::spawn;

// This is a slow test.
#[ignore]
#[test]
fn test_magics() {
    let t = spawn(|| compute_bmagics());
    let r = compute_rmagics();

    let br = t.join();
    assert!(br.is_ok());

    let b = br.unwrap();
    assert!(b.is_ok());
    assert!(r.is_ok());

    let b = b.unwrap();
    let r = r.unwrap();

    let fb = from_bmagics(b.as_ref().iter().map(|m| m.magic));
    let fr = from_rmagics(r.as_ref().iter().map(|m| m.magic));

    assert!(fb.is_ok());
    assert!(fr.is_ok());

    let fb = fb.unwrap();
    let fr = fr.unwrap();

    assert_eq!(b, fb);
    assert_eq!(r, fr);
}
