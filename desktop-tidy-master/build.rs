use embed_resource::CompilationResult;

fn main() {
    assert!(
        embed_resource::compile("res/icon.rc", embed_resource::NONE) == CompilationResult::Ok,
        "Compiling icon.rc failed!"
    );
}
