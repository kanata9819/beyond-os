use std::path::PathBuf;

use anyhow::Context;
use bootloader::BiosBoot;

fn main() -> anyhow::Result<()> {
    // このバイナリはビルド済みカーネルELFからBIOS起動用のディスクイメージを生成する。
    // 以降ではプロジェクトルートを起点にパスを組み立てる。
    // プロジェクトルートへのパス
    let root: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..");

    // ① カーネルELFのパス
    //   Cargoのターゲット出力先を固定で想定している。
    //   例: target/x86_64-unknown-none/debug/kernel
    let kernel_path = root
        .join("target")
        .join("x86_64-unknown-none")
        .join("debug")
        .join("kernel");

    // 期待した場所にカーネルが無ければ、作業者にビルド手順を案内して終了する。
    if !kernel_path.exists() {
        eprintln!("kernel binary not found at {}", kernel_path.display());
        eprintln!("先にこれを実行した？:");
        eprintln!("  cargo +nightly build -p kernel --target x86_64-unknown-none");
        std::process::exit(1);
    }

    // ② 出力するBIOSブート用ディスクイメージのパス
    //   既存のファイルがあれば上書きされる。
    let out_path: PathBuf = root.join("target").join("bios.img");

    // ③ BIOS用ディスクイメージを作成
    //   bootloaderクレートがBIOSブート可能な形式に変換してくれる。
    BiosBoot::new(&kernel_path)
        .create_disk_image(&out_path)
        .with_context(|| format!("failed to create disk image at {}", out_path.display()))?;

    // 完了メッセージ。生成先を明示しておくと次の工程がわかりやすい。
    println!("✅ Created BIOS image at {}", out_path.display());

    Ok(())
}
