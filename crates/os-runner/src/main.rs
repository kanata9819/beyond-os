use std::path::PathBuf;

use anyhow::Context;
use bootloader::BiosBoot;

fn main() -> anyhow::Result<()> {
    // プロジェクトルートへのパス
    let root: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..");

    // ① カーネル ELF のパス
    //   → さっきのビルドで出来たやつ
    //   target/x86_64-unknown-none/debug/kernel になってるはず
    let kernel_path = root
        .join("target")
        .join("x86_64-unknown-none")
        .join("debug")
        .join("kernel");

    if !kernel_path.exists() {
        eprintln!("kernel binary not found at {}", kernel_path.display());
        eprintln!("先にこれを実行した？:");
        eprintln!("  cargo +nightly build -p kernel --target x86_64-unknown-none");
        std::process::exit(1);
    }

    // ② 出力する BIOS イメージのパス
    let out_path: PathBuf = root.join("target").join("bios.img");

    // ③ BIOS 用ディスクイメージを作成
    BiosBoot::new(&kernel_path)
        .create_disk_image(&out_path)
        .with_context(|| format!("failed to create disk image at {}", out_path.display()))?;

    println!("✅ Created BIOS image at {}", out_path.display());

    Ok(())
}
