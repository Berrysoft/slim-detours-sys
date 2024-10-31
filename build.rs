fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let files = [
        "Disassembler.c",
        "FuncTableHook.c",
        "InlineHook.c",
        "Instruction.c",
        "Memory.c",
        "Thread.c",
        "Trampoline.c",
        "Transaction.c",
    ];
    let mut build = cc::Build::new();

    build
        .include("third-party/KNSoft.NDK/Source/Include")
        .define("_USE_KNSOFT_NDK", "1")
        .define("NDEBUG", "1")
        .files(
            files
                .iter()
                .map(|name| format!("third-party/KNSoft.SlimDetours/Source/SlimDetours/{}", name)),
        )
        .cpp(false)
        .compile("KNSoft.SlimDetours");
}
