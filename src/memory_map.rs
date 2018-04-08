use core::ops::{Deref, DerefMut};
use x86_64::PhysAddr;

#[derive(Debug)]
#[repr(C)]
pub struct MemoryMap {
    entries: [MemoryRegion; 32],
    // u64 instead of usize so that the structure layout is platform
    // independent
    next_entry_index: u64,
}

impl MemoryMap {
    pub fn new() -> Self {
        MemoryMap {
            entries: [MemoryRegion::empty(); 32],
            next_entry_index: 0,
        }
    }

    pub fn add_region(&mut self, region: MemoryRegion) {
        self.entries[self.next_entry_index()] = region;
        self.next_entry_index += 1;
    }

    pub fn sort(&mut self) {
        use core::cmp::Ordering;

        self.entries.sort_unstable_by(|r1, r2|
            if r1.len == 0 {
                Ordering::Greater
            } else if r2.len == 0 {
                Ordering::Less
            } else {
                r1.start_addr.cmp(&r2.start_addr)
            }
        );
        if let Some(first_zero_index) = self.entries.iter().position(|r| r.len == 0) {
            self.next_entry_index = first_zero_index as u64;
        }
    }

    fn next_entry_index(&self) -> usize {
        self.next_entry_index as usize
    }
}

impl Deref for MemoryMap {
    type Target = [MemoryRegion];

    fn deref(&self) -> &Self::Target {
        &self.entries[0..self.next_entry_index()]
    }
}

impl DerefMut for MemoryMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let next_index = self.next_entry_index();
        &mut self.entries[0..next_index]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct MemoryRegion {
    pub start_addr: PhysAddr,
    pub len: u64,
    pub region_type: MemoryRegionType
}

impl MemoryRegion {
    pub fn empty() -> Self {
        MemoryRegion {
            start_addr: PhysAddr::new(0),
            len: 0,
            region_type: MemoryRegionType::Reserved,
        }
    }

    pub fn start_addr(&self) -> PhysAddr {
        self.start_addr
    }

    pub fn end_addr(&self) -> PhysAddr {
        self.start_addr + self.len
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum MemoryRegionType {
    /// free RAM
    Usable,
    /// used RAM
    InUse,
    /// unusable
    Reserved,
    /// ACPI reclaimable memory
    AcpiReclaimable,
    /// ACPI NVS memory
    AcpiNvs,
    /// Area containing bad memory
    BadMemory,
    /// kernel memory
    Kernel,
    /// memory used by page tables
    PageTable,
    /// memory used by the bootloader
    Bootloader,
    /// frame at address zero
    ///
    /// (shouldn't be used because it's easy to make mistakes related to null pointers)
    FrameZero,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct E820MemoryRegion {
    pub start_addr: u64,
    pub len: u64,
    pub region_type: u32,
    pub acpi_extended_attributes: u32,
}

impl From<E820MemoryRegion> for MemoryRegion {
    fn from(region: E820MemoryRegion) -> MemoryRegion {
        let region_type = match region.region_type {
            1 => MemoryRegionType::Usable,
            2 => MemoryRegionType::Reserved,
            3 => MemoryRegionType::AcpiReclaimable,
            4 => MemoryRegionType::AcpiNvs,
            5 => MemoryRegionType::BadMemory,
            t => panic!("invalid region type {}", t),
        };
        MemoryRegion {
            start_addr: PhysAddr::new(region.start_addr),
            len: region.len,
            region_type
        }
    }
}

extern "C" {
    fn _improper_ctypes_check(_boot_info: MemoryMap);
}
