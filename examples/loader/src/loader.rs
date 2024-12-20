use alloc::vec;
use alloc::vec::Vec;
use axhal::mem::{virt_to_phys, MemoryAddr, VirtAddr, PAGE_SIZE_4K};
use axhal::paging::MappingFlags;
use axlog::warn;
use axmm::{kernel_aspace, AddrSpace};
use std::fs::File;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::{self, Read};

use elf::abi::{PT_INTERP, PT_LOAD};
use elf::endian::AnyEndian;
use elf::parse::ParseAt;
use elf::segment::ProgramHeader;
use elf::segment::SegmentTable;
use elf::ElfBytes;

const ELF_HEAD_BUF_SIZE: usize = 256;
//[PA:0x802a7000, PA:0x88000000]
const VA_BASE: usize = 0xffffffc000000000;
const PA_BASE: usize = 0x8100_0000;
const PA_STACK_BASE: usize = 0x8600_0000;

pub fn load_user_app(fname: &str, kspace: &mut AddrSpace) -> io::Result<usize> {
    let mut file = File::open(fname)?;
    let (phdrs, entry, _, _) = load_elf_phdrs(&mut file)?;

    // let mut kspace = kernel_aspace().lock();
    for phdr in &phdrs {
        warn!(
            "phdr: offset: {:#X}=>{:#X} size: {:#X}=>{:#X}",
            phdr.p_offset, phdr.p_vaddr, phdr.p_filesz, phdr.p_memsz
        );

        //APP_START_OFFSET as u64 +
        let vaddr_begin = phdr.p_vaddr;
        let vaddr = VirtAddr::from(vaddr_begin as usize).align_down_4k();
        let vaddr_end = VirtAddr::from((vaddr_begin + phdr.p_memsz) as usize).align_up_4k();
        // let (pha, _, _) = kspace
        //     .page_table()
        //     .query((VA_BASE + vaddr.as_usize()).into())
        //     .unwrap();
        warn!(
            "remap {:?} map_linear {:?} - with EXECUTE , size {}",
            kspace,
            vaddr,
            // pha,
            vaddr_end - vaddr
        );

        // kspace.unmap(vaddr, vaddr_end - vaddr)?;
        kspace.map_linear(
            (VA_BASE + vaddr.as_usize()).into(),
            (PA_BASE + vaddr.as_usize()).into(),
            vaddr_end - vaddr,
            MappingFlags::READ | MappingFlags::WRITE | MappingFlags::EXECUTE, //| MappingFlags::USER,
                                                                              // true,
        )?;

        let mut data = vec![0u8; phdr.p_memsz as usize];
        file.seek(SeekFrom::Start(phdr.p_offset))?;

        let filesz = phdr.p_filesz as usize;
        let mut index = 0;
        while index < filesz {
            let n = file.read(&mut data[index..filesz])?;
            index += n;
        }
        assert_eq!(index, filesz);
        warn!(
            "phdr: offset: {:#X}=>{:#X} size: {:#X}=>{:#X}",
            phdr.p_offset, phdr.p_vaddr, phdr.p_filesz, phdr.p_memsz
        );
        let vd = VirtAddr::from(vaddr_begin as usize + VA_BASE);
        warn!("kernel_aspace write ph data from :{:?} {},", vd, data.len());
        kspace.write(vd, &data)?;
    }

    Ok(entry + VA_BASE)
}

fn load_elf_phdrs(file: &mut File) -> io::Result<(Vec<ProgramHeader>, usize, usize, usize)> {
    let mut buf: [u8; ELF_HEAD_BUF_SIZE] = [0; ELF_HEAD_BUF_SIZE];
    file.read(&mut buf)?;

    let ehdr = ElfBytes::<AnyEndian>::parse_elf_header(&buf[..]).unwrap();
    warn!("e_entry: {:#X}", ehdr.e_entry);

    let phnum = ehdr.e_phnum as usize;
    // Validate phentsize before trying to read the table so that we can error early for corrupted files
    let entsize = ProgramHeader::validate_entsize(ehdr.class, ehdr.e_phentsize as usize).unwrap();
    let size = entsize.checked_mul(phnum).unwrap();
    assert!(size > 0 && size <= PAGE_SIZE_4K);
    let phoff = ehdr.e_phoff;
    let mut buf = alloc::vec![0u8; size];
    let _ = file.seek(SeekFrom::Start(phoff));
    file.read(&mut buf)?;
    let phdrs = SegmentTable::new(ehdr.endianness, ehdr.class, &buf[..]);

    let phdrs: Vec<ProgramHeader> = phdrs
        .iter()
        .filter(|phdr| phdr.p_type == PT_LOAD || phdr.p_type == PT_INTERP)
        .collect();
    Ok((
        phdrs,
        ehdr.e_entry as usize,
        ehdr.e_phoff as usize,
        ehdr.e_phnum as usize,
    ))
}

const KERNEL_STACK_SIZE: usize = 0x40000; // 256 KiB

pub fn init_app_stack(uspace: &mut AddrSpace) -> io::Result<VirtAddr> {
    let ustack_top: VirtAddr = uspace.end();
    let ustack_vaddr = ustack_top - KERNEL_STACK_SIZE;
    warn!(
        "Use exists Mapping APP stack: {:#x?} -> {:#x?}",
        ustack_vaddr, ustack_top
    );
    // failed to map page: 0xffffffc080fb0000(Size4K) -> PA:0x80fb0000, AlreadyMapped
    // [  0.509538 0:2 axmm:23] Paging error: AlreadyMapped
    uspace
        .map_linear(
            ustack_vaddr,
            PA_STACK_BASE.into(),
            KERNEL_STACK_SIZE,
            MappingFlags::READ | MappingFlags::WRITE,
        )
        .unwrap();

    let app_name = "hello";
    let av = std::collections::btree_map::BTreeMap::new();
    let (stack_data, ustack_pointer) = kernel_elf_parser::get_app_stack_region(
        &[alloc::string::String::from(app_name)],
        &[],
        &av,
        ustack_vaddr,
        KERNEL_STACK_SIZE,
    );
    // warn!(
    //     "Mapping ustack_pointer: {:#x?} ,stack_data_size {} ",
    //     ustack_pointer,
    //     stack_data.len()
    // );
    uspace.write(VirtAddr::from_usize(ustack_pointer), stack_data.as_slice())?;
    warn!(
        "[APP]  Mapping & Write ustack_pointer: {:#x?} ,stack_data_size {} ",
        ustack_pointer,
        stack_data.len()
    );
    Ok(ustack_pointer.into())
}
