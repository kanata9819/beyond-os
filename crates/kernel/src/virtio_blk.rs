use core::mem::{align_of, size_of};
use core::ptr;
use core::sync::atomic::{Ordering, fence};

use memory::align_up_usize;
use x86_64::instructions::port::Port;

const QUEUE_SIZE: u16 = 8;
const SECTOR_SIZE: usize = 512;

const VIRTQ_DESC_F_NEXT: u16 = 1;
const VIRTQ_DESC_F_WRITE: u16 = 2;

const VIRTIO_BLK_T_IN: u32 = 0;
#[allow(dead_code)]
const VIRTIO_BLK_T_OUT: u32 = 1;

const STATUS_ACK: u8 = 0x01;
const STATUS_DRIVER: u8 = 0x02;
const STATUS_DRIVER_OK: u8 = 0x04;

const REG_HOST_FEATURES: u16 = 0x00;
const REG_GUEST_FEATURES: u16 = 0x04;
const REG_QUEUE_PFN: u16 = 0x08;
const REG_QUEUE_NUM: u16 = 0x0c;
const REG_QUEUE_SEL: u16 = 0x0e;
const REG_QUEUE_NOTIFY: u16 = 0x10;
const REG_STATUS: u16 = 0x12;
const REG_CONFIG: u16 = 0x14;

const QUEUE_INDEX: u16 = 0;
const FEATURES_NONE: u32 = 0;
const STATUS_RESET: u8 = 0x00;
const QUEUE_UNAVAILABLE: u16 = 0;
const INITIAL_USED_IDX: u16 = 0;

const DESC_HEADER_INDEX: usize = 0;
const DESC_DATA_INDEX: usize = 1;
const DESC_STATUS_INDEX: usize = 2;
const DESC_STATUS_LEN: u32 = 1;
const DESC_CHAIN_END: u16 = 0;
const DESC_FLAGS_NONE: u16 = 0;

const AVAIL_RING_ENTRY_SIZE: usize = size_of::<u16>();
const AVAIL_USED_EVENT_SIZE: usize = size_of::<u16>();
const USED_EVENT_SIZE: usize = size_of::<u16>();

const REQUEST_STATUS_PENDING: u8 = 0xff;
const REQUEST_STATUS_TIMEOUT: u8 = 0xfe;
const REQUEST_STATUS_OK: u8 = 0x00;
const REQUEST_TIMEOUT_SPINS: u64 = 5_000_000;
const SPIN_INCREMENT: u64 = 1;
const IDX_INCREMENT: u16 = 1;

const CONFIG_CAPACITY_HIGH_OFFSET: u16 = 4;
const CAPACITY_HIGH_SHIFT: u32 = 32;

#[repr(C, align(16))]
struct VirtqDesc {
    addr: u64,
    len: u32,
    flags: u16,
    next: u16,
}

#[repr(C)]
struct VirtqAvailHeader {
    flags: u16,
    idx: u16,
}

#[repr(C)]
struct VirtqUsedElem {
    id: u32,
    len: u32,
}

#[repr(C)]
struct VirtqUsedHeader {
    flags: u16,
    idx: u16,
}

#[repr(C)]
struct VirtioBlkReq {
    req_type: u32,
    reserved: u32,
    sector: u64,
}

const REQ_RESERVED: u32 = 0;
const REQ_DATA_OFFSET: usize = size_of::<VirtioBlkReq>();
const REQ_STATUS_OFFSET: usize = size_of::<VirtioBlkReq>() + SECTOR_SIZE;
const ZERO_FILL: u8 = 0;

pub struct VirtioBlk {
    io_base: u16,
    queue_size: u16,
    #[allow(dead_code)]
    queue_paddr: u64,
    desc: *mut VirtqDesc,
    avail: *mut VirtqAvailHeader,
    used: *mut VirtqUsedHeader,
    last_used_idx: u16,
    req_paddr: u64,
    req_vaddr: *mut u8,
    capacity_sectors: u64,
}

impl VirtioBlk {
    pub fn capacity_sectors(&self) -> u64 {
        self.capacity_sectors
    }

    pub fn read_sector(
        &mut self,
        sector: u64,
        out: &mut [u8; SECTOR_SIZE],
    ) -> Result<(), &'static str> {
        unsafe {
            self.submit_request(VIRTIO_BLK_T_IN, sector)?;
            let data_ptr = self.req_vaddr.add(REQ_DATA_OFFSET);
            ptr::copy_nonoverlapping(data_ptr, out.as_mut_ptr(), SECTOR_SIZE);
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn write_sector(
        &mut self,
        sector: u64,
        data: &[u8; SECTOR_SIZE],
    ) -> Result<(), &'static str> {
        unsafe {
            let data_ptr = self.req_vaddr.add(REQ_DATA_OFFSET);
            ptr::copy_nonoverlapping(data.as_ptr(), data_ptr, SECTOR_SIZE);
            self.submit_request(VIRTIO_BLK_T_OUT, sector)?;
        }
        Ok(())
    }

    unsafe fn submit_request(&mut self, req_type: u32, sector: u64) -> Result<(), &'static str> {
        let status = unsafe {
            let header_ptr = self.req_vaddr as *mut VirtioBlkReq;
            (*header_ptr).req_type = req_type;
            (*header_ptr).reserved = REQ_RESERVED;
            (*header_ptr).sector = sector;

            let status_ptr = self.req_vaddr.add(REQ_STATUS_OFFSET);
            *status_ptr = REQUEST_STATUS_PENDING;

            let desc = core::slice::from_raw_parts_mut(self.desc, self.queue_size as usize);
            desc[DESC_HEADER_INDEX] = VirtqDesc {
                addr: self.req_paddr,
                len: size_of::<VirtioBlkReq>() as u32,
                flags: VIRTQ_DESC_F_NEXT,
                next: DESC_DATA_INDEX as u16,
            };
            desc[DESC_DATA_INDEX] = VirtqDesc {
                addr: self.req_paddr + size_of::<VirtioBlkReq>() as u64,
                len: SECTOR_SIZE as u32,
                flags: VIRTQ_DESC_F_NEXT
                    | if req_type == VIRTIO_BLK_T_IN {
                        VIRTQ_DESC_F_WRITE
                    } else {
                        DESC_FLAGS_NONE
                    },
                next: DESC_STATUS_INDEX as u16,
            };
            desc[DESC_STATUS_INDEX] = VirtqDesc {
                addr: self.req_paddr + (size_of::<VirtioBlkReq>() + SECTOR_SIZE) as u64,
                len: DESC_STATUS_LEN,
                flags: VIRTQ_DESC_F_WRITE,
                next: DESC_CHAIN_END,
            };

            let avail = &mut *self.avail;
            let ring_index = (avail.idx % self.queue_size) as usize;
            let ring_ptr = (self.avail as *mut u8)
                .add(size_of::<VirtqAvailHeader>() + ring_index * AVAIL_RING_ENTRY_SIZE)
                as *mut u16;
            ptr::write_volatile(ring_ptr, DESC_HEADER_INDEX as u16);
            fence(Ordering::SeqCst);
            avail.idx = avail.idx.wrapping_add(IDX_INCREMENT);

            io_write_u16(self.io_base, REG_QUEUE_NOTIFY, QUEUE_INDEX);

            let used = &mut *self.used;
            let mut spins = 0u64;
            while ptr::read_volatile(&used.idx) == self.last_used_idx {
                core::hint::spin_loop();
                spins = spins.wrapping_add(SPIN_INCREMENT);
                if spins == REQUEST_TIMEOUT_SPINS {
                    break;
                }
            }
            fence(Ordering::SeqCst);
            if used.idx == self.last_used_idx {
                REQUEST_STATUS_TIMEOUT
            } else {
                self.last_used_idx = self.last_used_idx.wrapping_add(IDX_INCREMENT);
                *status_ptr
            }
        };

        if status == REQUEST_STATUS_TIMEOUT {
            return Err("virtio-blk request timed out");
        }
        if status != REQUEST_STATUS_OK {
            return Err("virtio-blk request failed");
        }

        Ok(())
    }
}

pub fn init_legacy(io_base: u16, phys_offset: u64) -> Result<VirtioBlk, &'static str> {
    // Reset device status, then acknowledge and announce the driver.
    io_write_u8(io_base, REG_STATUS, STATUS_RESET);
    io_write_u8(io_base, REG_STATUS, STATUS_ACK);
    io_write_u8(io_base, REG_STATUS, STATUS_ACK | STATUS_DRIVER);

    // Read and ignore host features for now, then advertise none.
    let _host_features = io_read_u32(io_base, REG_HOST_FEATURES);
    io_write_u32(io_base, REG_GUEST_FEATURES, FEATURES_NONE);

    // Select queue 0 and read its max size.
    io_write_u16(io_base, REG_QUEUE_SEL, QUEUE_INDEX);
    let max_queue = io_read_u16(io_base, REG_QUEUE_NUM);
    if max_queue == QUEUE_UNAVAILABLE {
        return Err("virtio-blk: queue 0 not available");
    }
    let queue_size = core::cmp::min(max_queue, QUEUE_SIZE);
    io_write_u16(io_base, REG_QUEUE_NUM, queue_size);

    // Allocate a page for the virtqueue and set the queue PFN.
    let queue_paddr = memory::alloc_frame().ok_or("virtio-blk: no frames")?;
    let queue_vaddr = phys_offset + queue_paddr;
    unsafe {
        ptr::write_bytes(
            queue_vaddr as *mut u8,
            ZERO_FILL,
            memory::PAGE_SIZE as usize,
        )
    };

    let desc_size = size_of::<VirtqDesc>() * queue_size as usize;
    let avail_offset = align_up_usize(desc_size, align_of::<VirtqAvailHeader>());
    let avail_size = size_of::<VirtqAvailHeader>()
        + (AVAIL_RING_ENTRY_SIZE * queue_size as usize)
        + AVAIL_USED_EVENT_SIZE;
    let used_offset = align_up_usize(avail_offset + avail_size, align_of::<VirtqUsedHeader>());
    let used_size = size_of::<VirtqUsedHeader>()
        + (size_of::<VirtqUsedElem>() * queue_size as usize)
        + USED_EVENT_SIZE;

    let total = used_offset + used_size;
    if total > memory::PAGE_SIZE as usize {
        return Err("virtio-blk: queue layout exceeds one page");
    }

    let desc_ptr = queue_vaddr as *mut VirtqDesc;
    let avail_ptr = (queue_vaddr + avail_offset as u64) as *mut VirtqAvailHeader;
    let used_ptr = (queue_vaddr + used_offset as u64) as *mut VirtqUsedHeader;

    let queue_pfn = queue_paddr / memory::PAGE_SIZE;
    io_write_u32(io_base, REG_QUEUE_PFN, queue_pfn as u32);

    // Allocate one page for request header + data + status.
    let req_paddr = memory::alloc_frame().ok_or("virtio-blk: no frames")?;
    let req_vaddr = (phys_offset + req_paddr) as *mut u8;
    unsafe { ptr::write_bytes(req_vaddr, ZERO_FILL, memory::PAGE_SIZE as usize) };

    // Read capacity (in 512-byte sectors) from the device-specific config.
    let cap_low = io_read_u32(io_base, REG_CONFIG);
    let cap_high = io_read_u32(io_base, REG_CONFIG + CONFIG_CAPACITY_HIGH_OFFSET);
    let capacity_sectors = ((cap_high as u64) << CAPACITY_HIGH_SHIFT) | cap_low as u64;

    io_write_u8(
        io_base,
        REG_STATUS,
        STATUS_ACK | STATUS_DRIVER | STATUS_DRIVER_OK,
    );

    Ok(VirtioBlk {
        io_base,
        queue_size,
        queue_paddr,
        desc: desc_ptr,
        avail: avail_ptr,
        used: used_ptr,
        last_used_idx: INITIAL_USED_IDX,
        req_paddr,
        req_vaddr,
        capacity_sectors,
    })
}

#[allow(dead_code)]
fn io_read_u8(base: u16, offset: u16) -> u8 {
    unsafe { Port::<u8>::new(base + offset).read() }
}

fn io_read_u16(base: u16, offset: u16) -> u16 {
    unsafe { Port::<u16>::new(base + offset).read() }
}

fn io_read_u32(base: u16, offset: u16) -> u32 {
    unsafe { Port::<u32>::new(base + offset).read() }
}

fn io_write_u8(base: u16, offset: u16, value: u8) {
    unsafe { Port::<u8>::new(base + offset).write(value) }
}

fn io_write_u16(base: u16, offset: u16, value: u16) {
    unsafe { Port::<u16>::new(base + offset).write(value) }
}

fn io_write_u32(base: u16, offset: u16, value: u32) {
    unsafe { Port::<u32>::new(base + offset).write(value) }
}
