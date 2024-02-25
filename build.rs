extern crate embed_resource;
fn main() {
    embed_resource::compile("passtool-manifest.rc", embed_resource::NONE);
}