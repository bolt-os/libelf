/*
 * Copyright (c) 2022 xvanc and contributors
 *
 * Redistribution and use in source and binary forms, with or without modification,
 * are permitted provided that the following conditions are met:
 *
 * 1. Redistributions of source code must retain the above copyright notice,
 *    this list of conditions and the following disclaimer.
 *
 * 2. Redistributions in binary form must reproduce the above copyright notice,
 *    this list of conditions and the following disclaimer in the documentation
 *    and/or other materials provided with the distribution.
 *
 * 3. Neither the name of the copyright holder nor the names of its contributors
 *    may be used to endorse or promote products derived from this software without
 *    specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES
 * OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED.
 * IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT,
 * INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
 * PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
 * INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT
 * LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 *
 * SPDX-License-Identifier: BSD-3-Clause
 */

//! Executable and Linkable Format (ELF)
//!
//! This crate is implemented based on [this draft] of the System V ABI specification.
//!
//! [this draft]: https://refspecs.linuxfoundation.org/elf/gabi4+/contents.html

#![feature(const_align_offset)]
#![no_std]

mod dynamic;
mod reloc;
mod section;
mod segment;
mod symbol;
mod types;

pub use dynamic::*;
pub use reloc::*;
pub use section::*;
pub use segment::*;
pub use symbol::*;
pub use types::*;

macro_rules! assert_struct_size {
    ($struc:ty, $size:expr) => {
        const _: () = {
            ::core::assert!(::core::mem::size_of::<$struc>() == $size);
        };
    };
}
pub(crate) use assert_struct_size;

unsafe fn strlen(s: *const u8) -> usize {
    let mut len = 0;

    while *s.add(len) != 0 {
        len += 1;
    }

    len
}

use core::{fmt, mem::size_of, ops::Deref};

#[derive(Debug)]
pub struct StringTable<'elf> {
    table: &'elf [u8],
}

impl StringTable<'_> {
    pub const fn len(&self) -> usize {
        self.table.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub enum StrtabResult<'a> {
    Ok(&'a str),
    Invalid(&'a [u8]),
    OutOfBounds,
}

impl<'elf> StringTable<'elf> {
    pub fn new(table: &'elf [u8]) -> StringTable<'elf> {
        Self { table }
    }

    pub fn get_slice(&self, index: usize) -> Option<&'elf [u8]> {
        if index >= self.len() {
            return None;
        }

        let buf = &self.table[index..];
        let len = unsafe { strlen(buf.as_ptr()) };

        Some(&buf[..len])
    }

    /// Get the string at the provided index.
    pub fn get_string(&self, index: usize) -> Option<&'elf str> {
        if index < self.len() {
            let buf = &self.table[index..];
            let mut len = 0;
            while buf[len] != 0 {
                len += 1;
            }
            Some(core::str::from_utf8(&buf[..len]).unwrap())
        } else {
            None
        }
    }
}

pub struct Elf<'elf> {
    data: &'elf [u8],
    ehdr: &'elf FileHeader,
}

impl Elf<'_> {
    pub const fn new(data: &[u8]) -> Result<Elf<'_>, &'static str> {
        if !FileHeader::check_buffer(data) {
            return Err("invalid ELF");
        }

        let ehdr = FileHeader::from_buffer(data);

        if ehdr.class != Class::Bits64.to_u8() {
            return Err("not ELF64");
        }

        if ehdr.phdr_size as usize != size_of::<ProgramHeader>() {
            return Err("bad program header size");
        }
        if ehdr.shdr_size as usize != size_of::<SectionHeader>() {
            return Err("bad section header size");
        }

        Ok(Elf { data, ehdr })
    }

    fn get_slice(&self, offset: usize, size: usize) -> &[u8] {
        &self.data[offset..][..size]
    }

    unsafe fn get_slice_of<T>(&self, offset: usize, size: usize) -> &[T] {
        let buf = &self.data[offset..][..size];
        let data = buf.as_ptr().cast::<T>();
        let len = size / size_of::<T>();

        core::slice::from_raw_parts(data, len)
    }

    pub fn section_string_table(&self) -> Option<StringTable<'_>> {
        let shdr = self.section_header(self.ehdr.shdr_strtab_index as _)?;

        Some(StringTable {
            table: self.get_slice(shdr.file_offset(), shdr.size()),
        })
    }

    pub fn string_table(&self) -> Option<StringTable<'_>> {
        let shdr = self
            .sections()
            .find(|shdr| shdr.name() == Some(".strtab"))?;

        Some(StringTable {
            table: self.get_slice(shdr.file_offset(), shdr.size()),
        })
    }

    pub fn symtab(&self) -> Option<impl Iterator<Item = &Sym>> {
        let shdr = self.sections().find(|s| s.name() == Some(".symtab"))?;
        assert!(shdr.entry_size() as usize == size_of::<Sym>());
        let symtab_len = shdr.size() / size_of::<Sym>();
        let table = self.data[shdr.file_offset()..].as_ptr().cast::<Sym>();
        let mut index = 0;

        Some(core::iter::from_fn(move || {
            if index < symtab_len {
                let hdr = unsafe { &*table.add(index) };
                index += 1;
                Some(hdr)
            } else {
                None
            }
        }))
    }

    pub fn symbol_table(&self) -> Option<SymbolTable<'_>> {
        let shdr = self
            .sections()
            .find(|shdr| shdr.name() == Some(".strtab"))?;

        Some(SymbolTable::new(
            self,
            unsafe { self.get_slice_of(shdr.file_offset(), shdr.size()) },
            self.string_table(),
        ))
    }

    pub fn section_header(&self, index: u32) -> Option<&SectionHeader> {
        self.section_headers().nth(index as usize)
    }

    pub fn sections(&self) -> impl Iterator<Item = Section<'_>> {
        self.section_headers().map(|hdr| Section::new(self, hdr))
    }

    pub fn segments(&self) -> impl Iterator<Item = Segment<'_>> {
        self.program_headers().map(|hdr| Segment::new(self, hdr))
    }

    pub fn program_headers(&self) -> impl Iterator<Item = &ProgramHeader> {
        let table = self.data[self.phdr_offset()..]
            [..self.phdr_num as usize * size_of::<ProgramHeader>()]
            .as_ptr()
            .cast::<ProgramHeader>();
        let mut index = 0;

        core::iter::from_fn(move || {
            if index < self.ehdr.phdr_num as usize {
                let hdr = unsafe { &*table.add(index) };
                index += 1;
                Some(hdr)
            } else {
                None
            }
        })
    }

    pub fn section_headers(&self) -> impl Iterator<Item = &SectionHeader> {
        let table = self.data[self.shdr_offset() as usize..]
            [..self.shdr_num as usize * size_of::<SectionHeader>()]
            .as_ptr()
            .cast::<SectionHeader>();
        let mut index = 0;

        core::iter::from_fn(move || {
            if index < self.ehdr.shdr_num as usize {
                let hdr = unsafe { &*table.add(index) };
                index += 1;
                Some(hdr)
            } else {
                None
            }
        })
    }

    pub fn section(&self, index: u16) -> Option<Section<'_>> {
        self.sections().nth(index as _)
    }

    pub fn dynamic_table(&self) -> Option<DynamicTable<'_>> {
        self.segments()
            .find(|sgmt| sgmt.kind() == SegmentKind::Dynamic)
            .map(|sgmt| {
                let data = &self.data[sgmt.file_offset()..][..sgmt.file_size()];
                DynamicTable::new(self, data)
            })
    }
}

impl fmt::Debug for Elf<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <FileHeader as fmt::Debug>::fmt(self.ehdr, f)
    }
}

impl Deref for Elf<'_> {
    type Target = FileHeader;

    fn deref(&self) -> &Self::Target {
        self.ehdr
    }
}

#[repr(C)]
pub struct FileHeader {
    magic: [u8; 4],
    class: u8,
    data: u8,
    header_version: u8,
    os_abi: u8,
    os_abi_version: u8,
    _padding: [u8; 7],
    elf_type: u16,
    machine: u16,
    version: u32,
    entry_point: u64,
    phdr_offset: u64,
    shdr_offset: u64,
    flags: u32,
    header_size: u16,
    /// Program header size, in bytes
    phdr_size: u16,
    /// Number of program headers
    phdr_num: u16,
    shdr_size: u16,
    shdr_num: u16,
    /// Index of the section header which describes the section name string table
    shdr_strtab_index: u16,
}

assert_struct_size!(FileHeader, 64);

impl FileHeader {
    pub const fn magic(&self) -> &[u8; 4] {
        &self.magic
    }

    pub const fn class(&self) -> Class {
        Class::from_u8(self.class)
    }

    pub const fn data(&self) -> Data {
        Data::from_u8(self.data)
    }

    pub const fn header_version(&self) -> Version {
        Version::from_u8(self.header_version)
    }

    pub const fn os_abi(&self) -> OsAbi {
        OsAbi::from_u8(self.os_abi)
    }

    pub const fn os_abi_version(&self) -> u8 {
        self.os_abi_version
    }

    pub const fn file_type(&self) -> ElfType {
        ElfType::from_u16(self.elf_type)
    }

    pub const fn machine(&self) -> Machine {
        Machine::from_u16(self.machine)
    }

    pub const fn file_version(&self) -> Version {
        Version::from_u32(self.version)
    }

    pub const fn entry_point(&self) -> u64 {
        self.entry_point
    }

    pub const fn phdr_offset(&self) -> usize {
        self.phdr_offset as _
    }

    pub const fn shdr_offset(&self) -> usize {
        self.shdr_offset as _
    }

    pub const fn check_buffer(buf: &[u8]) -> bool {
        buf.as_ptr().align_offset(core::mem::align_of::<Self>()) == 0
            && buf.len() >= core::mem::size_of::<Self>()
            && buf[0] == 0x7f
            && buf[1] == b'E'
            && buf[2] == b'L'
            && buf[3] == b'F'
    }

    pub const fn from_buffer(buf: &[u8]) -> &FileHeader {
        assert!(Self::check_buffer(buf));

        unsafe { &*buf.as_ptr().cast() }
    }
}

impl fmt::Debug for FileHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FileHeader")
            .field("magic", &self.magic)
            .field("class", &self.class)
            .field("data", &self.data)
            .field("header_version", &self.header_version())
            .field("os_abi", &self.os_abi)
            .field("os_abi_version", &self.os_abi_version)
            .field("elf_type", &self.file_type())
            .field("machine", &self.machine())
            .field("version", &self.file_version())
            .field("entry_point", &format_args!("{:#018x}", self.entry_point))
            .field("phdrs_offset", &format_args!("{:#x}", self.phdr_offset))
            .field("shdrs_offset", &format_args!("{:#x}", self.shdr_offset))
            .field("flags", &format_args!("{:x}", self.flags))
            .field("header_size", &self.header_size)
            .field("phdr_size", &self.phdr_size)
            .field("phdr_num", &self.phdr_num)
            .field("shdr_size", &self.shdr_size)
            .field("shdr_num", &self.shdr_num)
            .field("shdr_strtab_index", &self.shdr_strtab_index)
            .finish()
    }
}
