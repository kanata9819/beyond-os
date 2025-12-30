set shell := ["powershell", "-NoLogo", "-NoProfile", "-Command"]
r:
    cargo build -p kernel --target x86_64-unknown-none
    cargo run -p os-runner
    qemu-system-x86_64 -drive file=target/bios.img,format=raw -drive file=target/data.img,format=raw,if=virtio -no-reboot -no-shutdown -serial stdio

rd:
    cargo build -p kernel --target x86_64-unknown-none
    cargo run -p os-runner
    qemu-system-x86_64 -drive file=target/bios.img,format=raw -drive file=target/data.img,format=raw,if=virtio -no-reboot -no-shutdown -serial stdio -s -S

lldb:
    & "C:\Program Files\LLVM\bin\lldb.exe" -s tools/lldb_qemu.lldb
