// SPDX-License-Identifier: GPL-3.0-or-later
fn main() {
    println!("cargo:rustc-link-arg-bins=-Wl,--export-dynamic-symbol=main");
}
