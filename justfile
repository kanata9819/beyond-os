set shell := ["powershell", "-NoLogo", "-NoProfile", "-Command"]
r:
    cargo build -p kernel --target x86_64-unknown-none
    cargo run -p os-runner
    qemu-system-x86_64 -drive file=target/bios.img,format=raw -no-reboot -no-shutdown -serial stdio
