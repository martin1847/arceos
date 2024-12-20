use alloc::vec;
use alloc::vec::Vec;
use axhal::mem::{virt_to_phys, MemoryAddr, VirtAddr, PAGE_SIZE_4K};
use axhal::paging::MappingFlags;
use axmm::AddrSpace;

use axstd::io;
use axstd::os::arceos::modules::axlog::warn;
use elf::abi::{PT_INTERP, PT_LOAD};
use elf::endian::AnyEndian;
use elf::parse::ParseAt;
use elf::segment::ProgramHeader;
use elf::segment::SegmentTable;
use elf::ElfBytes;

const ELF_HEAD_BUF_SIZE: usize = 256;

fn read_segment(data: &[u8], phdr: &ProgramHeader) -> Vec<u8> {
    let memsz = phdr.p_memsz as usize;
    let filesz = phdr.p_filesz as usize;
    let offset = phdr.p_offset as usize;

    // 创建一个足够大的向量来保存段数据
    let mut segment_data = vec![0u8; memsz];

    // 确保偏移量和文件大小在数据范围内
    if offset + filesz > data.len() {
        panic!("Segment offset and size exceed buffer bounds",);
    }

    // 从数据切片中读取文件内容到 segment_data
    segment_data[..filesz].copy_from_slice(&data[offset..offset + filesz]);

    segment_data
}

pub fn load_user_app(file_buf: &[u8], uspace: &mut AddrSpace) -> io::Result<usize> {
    // let mut file = File::open(fname)?;
    let (phdrs, entry, _, _) = load_elf_phdrs(file_buf)?;

    for phdr in &phdrs {
        warn!(
            "phdr: offset: {:#X}=>{:#X} size: {:#X}=>{:#X}",
            phdr.p_offset, phdr.p_vaddr, phdr.p_filesz, phdr.p_memsz
        );

        let vaddr = VirtAddr::from(phdr.p_vaddr as usize).align_down_4k();
        let vaddr_end = VirtAddr::from((phdr.p_vaddr + phdr.p_memsz) as usize).align_up_4k();

        // warn!("{:#x} - {:#x}", vaddr, vaddr_end);
        // uspace.map_linear(
        //     vaddr,
        //     virt_to_phys(vaddr),
        //     vaddr_end - vaddr,
        //     MappingFlags::READ | MappingFlags::WRITE | MappingFlags::EXECUTE, // true,
        // )?;

        // let mut data = vec![0u8; phdr.p_memsz as usize];
        // file.seek(SeekFrom::Start(phdr.p_offset))?;

        // let filesz = phdr.p_filesz as usize;
        // let mut index = 0;
        // while index < filesz {
        //     let n = file.read(&mut data[index..filesz])?;
        //     index += n;
        // }
        // assert_eq!(index, filesz);
        let data = read_segment(file_buf, phdr);
        warn!("load read_segment size {} ", data.len());
        // uspace.write(VirtAddr::from(phdr.p_vaddr as usize), &data)?;
    }

    Ok(entry)
}

pub fn load_elf_phdrs(file: &[u8]) -> io::Result<(Vec<ProgramHeader>, usize, usize, usize)> {
    // let mut buf: [u8; ELF_HEAD_BUF_SIZE] = [0; ELF_HEAD_BUF_SIZE];
    // file.read(&mut buf)?;

    let ehdr = ElfBytes::<AnyEndian>::parse_elf_header(&file[..(ELF_HEAD_BUF_SIZE)]).unwrap();
    warn!("e_entry : {:#X} , ehdr :{:?}", ehdr.e_entry, ehdr);

    let phnum = ehdr.e_phnum as usize;
    // Validate phentsize before trying to read the table so that we can error early for corrupted files
    let entsize = ProgramHeader::validate_entsize(ehdr.class, ehdr.e_phentsize as usize).unwrap();
    let size = entsize.checked_mul(phnum).unwrap();
    assert!(size > 0 && size <= PAGE_SIZE_4K);
    let phoff = ehdr.e_phoff as usize;
    // let mut buf = alloc::vec![0u8; size];
    // let _ = file.seek(SeekFrom::Start(phoff));
    // file.read(&mut buf)?;
    let phdrs = SegmentTable::new(ehdr.endianness, ehdr.class, &file[phoff..(size + phoff)]);

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
