// The stylesheet and script are baked into the binary with include_str!.
// Cargo does not watch those files on its own, so editing app.css without
// touching any .rs file would leave a stale binary emitting yesterday's CSS.
// Declaring them here makes a change to either one trigger a rebuild.

fn main() {
    println!("cargo:rerun-if-changed=web/app.css");
    println!("cargo:rerun-if-changed=web/app.js");
}
