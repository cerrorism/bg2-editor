//! Byte-level diff between two baldur.gam files, reporting contiguous
//! differing runs. Read-only.

use std::path::PathBuf;

fn main() {
    let mut args = std::env::args().skip(1);
    let a_path = PathBuf::from(args.next().expect("usage: diff_gam <a.gam> <b.gam>"));
    let b_path = PathBuf::from(args.next().expect("usage: diff_gam <a.gam> <b.gam>"));
    let a = std::fs::read(&a_path).expect("read a");
    let b = std::fs::read(&b_path).expect("read b");
    println!("a: {} bytes, b: {} bytes", a.len(), b.len());
    let min_len = a.len().min(b.len());
    let mut i = 0usize;
    let mut runs = 0;
    while i < min_len {
        if a[i] != b[i] {
            let start = i;
            while i < min_len && a[i] != b[i] {
                i += 1;
            }
            let len = i - start;
            println!("offset {start:#08x} ({start}), len {len}: a={:02x?} b={:02x?}", &a[start..i], &b[start..i]);
            runs += 1;
            if runs > 200 {
                println!("...(truncated)");
                break;
            }
        } else {
            i += 1;
        }
    }
    if a.len() != b.len() {
        println!("lengths differ: a={} b={}", a.len(), b.len());
    }
    println!("total diff runs: {runs}");
}
