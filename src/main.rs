use passtool::{PassTable, Error};
mod app;
fn main() {
    let pt = PassTable::new();
    app::run();
}
