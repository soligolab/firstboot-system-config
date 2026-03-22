fn main() {
    // La build genera in anticipo il codice Rust derivato dal file Slint, così il
    // crate principale può includerlo con `slint::include_modules!()`.
    slint_build::compile("ui/app.slint").expect("failed to compile Slint UI");
}
