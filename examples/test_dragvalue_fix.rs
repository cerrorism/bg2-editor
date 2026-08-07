//! Directly reproduces the bug reported against a real save: rendering a
//! DragValue bound to a pre-existing value outside its configured range
//! silently clamps that value (marking it "changed") the moment it's
//! drawn, with no user interaction. This headlessly renders the exact
//! widgets used in the Spells tab and the per-creature reputation field,
//! seeded with the exact real values that got corrupted (spell level 0,
//! reputation 120), across several frames, and asserts they now survive
//! unchanged with the fixed (widened) ranges — and, for contrast, proves
//! the bug is real by reproducing it with the old narrow ranges too.

fn render_frames(ctx: &egui::Context, mut body: impl FnMut(&mut egui::Ui)) {
    for _ in 0..3 {
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| body(ui));
        });
    }
}

fn main() {
    let ctx = egui::Context::default();

    // --- Reproduce the bug with the OLD (buggy) ranges ---
    let mut level_old: u16 = 0;
    render_frames(&ctx, |ui| {
        ui.add(egui::DragValue::new(&mut level_old).range(1..=9u16));
    });
    println!("old range(1..=9) starting from 0 -> {level_old} (bug reproduced: {})", level_old != 0);

    let mut reputation_old: u8 = 120;
    render_frames(&ctx, |ui| {
        ui.add(egui::DragValue::new(&mut reputation_old).range(0..=20u8));
    });
    println!("old range(0..=20) starting from 120 -> {reputation_old} (bug reproduced: {})", reputation_old != 120);

    // --- Verify the FIXED ranges leave real pre-existing values alone ---
    let mut level_new: u16 = 0;
    render_frames(&ctx, |ui| {
        ui.add(egui::DragValue::new(&mut level_new).range(0..=9u16));
    });
    println!("fixed range(0..=9) starting from 0 -> {level_new} (should stay 0)");

    let mut reputation_new: u8 = 120;
    render_frames(&ctx, |ui| {
        ui.add(egui::DragValue::new(&mut reputation_new).range(0..=255u8));
    });
    println!("fixed range(0..=255) starting from 120 -> {reputation_new} (should stay 120)");

    let mut resist_new: i8 = -25; // vulnerability
    render_frames(&ctx, |ui| {
        ui.add(egui::DragValue::new(&mut resist_new).range(i8::MIN..=i8::MAX));
    });
    println!("fixed i8 full range starting from -25 -> {resist_new} (should stay -25)");

    let ok = level_new == 0 && reputation_new == 120 && resist_new == -25;
    if ok {
        println!("\nFIX VERIFIED: real pre-existing values survive unchanged with the widened ranges.");
    } else {
        println!("\nFIX FAILED: a fixed-range field still got mutated just by rendering.");
        std::process::exit(1);
    }
}
