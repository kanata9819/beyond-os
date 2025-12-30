//! PCI 基礎メモ（最小限）
//! - PCI デバイスは「バス/デバイス/ファンクション(BDF)」で識別する。
//! - 設定空間は「Configuration Mechanism #1」で I/O ポート経由に読める。
//!   - 0xCF8: 設定アドレスレジスタ（どのBDF/オフセットを読むか指定）
//!   - 0xCFC: 設定データレジスタ（実際の 32bit データ）
//! - 0xCF8 に書くアドレスは次のビット配置：
//!   - bit31: Enable（1で有効）
//!   - bit23..16: bus
//!   - bit15..11: device
//!   - bit10..8 : function
//!   - bit7..2  : レジスタオフセット（32bit境界）
//! - まず function 0 を読んで存在確認（vendor_id == 0xFFFF なら未実装）。
//! - header_type の bit7 が 1 なら multi-function で 0..7 を走査する。
//! - class/subclass/prog-if/revision は offset 0x08 の 32bit に詰まっている。
//! - virtio は vendor_id 0x1AF4 で判別できる。
use x86_64::instructions::port::Port;

const CONFIG_ADDRESS: u16 = 0x0cf8;
const CONFIG_DATA: u16 = 0x0cfc;

#[derive(Debug, Clone, Copy)]
pub struct PciDevice {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class_code: u8,
    pub subclass: u8,
    pub prog_if: u8,
    pub revision_id: u8,
    pub header_type: u8,
}

/// PCI 設定空間を 0xCF8/0xCFC 経由で 32bit 読み出す。
/// BDF と 32bit 境界のオフセットをアドレス化して 0xCF8 に書き込み、
/// 0xCFC から実データを読む。
fn read_config_dword(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    // 0x8000_0000 は「設定空間アクセス有効」フラグ。
    let address = 0x8000_0000u32
        // bus を bit23..16 に埋め込む。
        | ((bus as u32) << 16)
        // device を bit15..11 に埋め込む。
        | ((device as u32) << 11)
        // function を bit10..8 に埋め込む。
        | ((function as u32) << 8)
        // offset を 32bit 境界（bit7..2）に丸める。
        | (offset as u32 & 0xfc);

    unsafe {
        let mut addr_port = Port::<u32>::new(CONFIG_ADDRESS);
        let mut data_port = Port::<u32>::new(CONFIG_DATA);
        addr_port.write(address);
        data_port.read()
    }
}

/// PCI 設定空間の 16bit 読み出し。
/// 32bit を読んでから offset の bit1 で下位/上位 16bit を選ぶ。
fn read_config_word(bus: u8, device: u8, function: u8, offset: u8) -> u16 {
    // 該当する 32bit を読み、その中から 16bit を切り出す。
    let value = read_config_dword(bus, device, function, offset & 0xfc);
    // offset の bit1 が 1 なら上位 16bit（16シフト）を選ぶ。
    let shift = (offset & 0x2) * 8;
    ((value >> shift) & 0xffff) as u16
}

/// offset 0x08 から class/subclass/prog-if/revision を読む。
/// 32bit の並びは [class][subclass][prog-if][revision]。
fn read_class_fields(bus: u8, device: u8, function: u8) -> (u8, u8, u8, u8) {
    // offset 0x08 は class/subclass/prog-if/revision が 1 ワードに詰まっている。
    let value = read_config_dword(bus, device, function, 0x08);
    let class_code = (value >> 24) as u8;
    let subclass = (value >> 16) as u8;
    let prog_if = (value >> 8) as u8;
    let revision_id = value as u8;
    (class_code, subclass, prog_if, revision_id)
}

/// offset 0x0C から header_type を読む（multi-function 判定に使う）。
fn read_header_type(bus: u8, device: u8, function: u8) -> u8 {
    // offset 0x0C の bit23..16 が header type。
    let value = read_config_dword(bus, device, function, 0x0c);
    ((value >> 16) & 0xff) as u8
}

/// 全バス/デバイス/ファンクションを走査し、存在するデバイスを callback に渡す。
/// function 0 で存在確認し、header_type の bit7 が 1 なら 0..7 を走査する。
pub fn scan(mut f: impl FnMut(PciDevice)) {
    for bus in 0u16..=255 {
        for device in 0u8..32 {
            let vendor_id = read_config_word(bus as u8, device, 0, 0x00);
            if vendor_id == 0xffff {
                continue;
            }

            let header_type = read_header_type(bus as u8, device, 0);
            let functions = if header_type & 0x80 != 0 { 8 } else { 1 };

            for function in 0u8..functions {
                let vendor_id = read_config_word(bus as u8, device, function, 0x00);
                if vendor_id == 0xffff {
                    continue;
                }

                let device_id = read_config_word(bus as u8, device, function, 0x02);
                let (class_code, subclass, prog_if, revision_id) =
                    read_class_fields(bus as u8, device, function);
                let header_type = read_header_type(bus as u8, device, function);

                f(PciDevice {
                    bus: bus as u8,
                    device,
                    function,
                    vendor_id,
                    device_id,
                    class_code,
                    subclass,
                    prog_if,
                    revision_id,
                    header_type,
                });
            }
        }
    }
}
