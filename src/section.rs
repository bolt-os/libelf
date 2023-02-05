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

use crate::{assert_struct_size, Elf};
use core::{
    fmt,
    mem::{align_of, size_of},
};

pub const SHN_UNDEF: u16 = 0;
pub const SHN_ABS: u16 = 0xfff1;
pub const SHN_COMMON: u16 = 0xfff2;
pub const SHN_XINDEX: u16 = 0xffff;

pub struct Section<'elf> {
    elf: &'elf Elf<'elf>,
    hdr: &'elf SectionHeader,
}

impl<'elf> Section<'elf> {
    #[inline]
    pub(crate) fn new(elf: &'elf Elf<'elf>, hdr: &'elf SectionHeader) -> Section<'elf> {
        Self { elf, hdr }
    }

    #[inline]
    pub fn file_data(&self) -> &'elf [u8] {
        &self.elf.data[self.file_offset()..][..self.size()]
    }

    /// Returns the contents of the section as an array of some type
    ///
    /// # Safety
    ///
    /// The caller must guarantee that the section's contents represent valid
    /// types of `T`.
    pub unsafe fn table<T>(&self) -> &'elf [T] {
        assert_eq!(self.entry_size as usize, size_of::<T>());
        let data = self.file_data().as_ptr().cast::<T>();
        assert!(data.align_offset(align_of::<T>()) == 0);
        assert!(self.size() % size_of::<T>() == 0);
        let len = self.size() / size_of::<T>();

        core::slice::from_raw_parts(data, len)
    }

    pub fn name(&self) -> Option<&'elf str> {
        let elf = self.elf;
        let string_table = elf.section_string_table()?;

        match self.name_index {
            0 => None,
            _ => string_table.get_string(self.name_index as _),
        }
    }
}

impl fmt::Debug for Section<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Section")
            .field("name", &self.name().unwrap_or(""))
            .field("type", &self.section_type())
            .finish()
    }
}

impl core::ops::Deref for Section<'_> {
    type Target = SectionHeader;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.hdr
    }
}

#[repr(C)]
pub struct SectionHeader {
    name_index: u32,
    section_type: u32,
    flags: u64,
    addr: u64,
    offset: u64,
    size: u64,
    link: u32,
    info: u32,
    addr_align: u64,
    entry_size: u64,
}

assert_struct_size!(SectionHeader, 64);

impl SectionHeader {
    #[inline]
    pub const fn name_index(&self) -> u32 {
        self.name_index
    }

    #[inline]
    pub const fn section_type(&self) -> SectionType {
        SectionType::from_u32(self.section_type)
    }

    #[inline]
    pub const fn flags(&self) -> SectionFlags {
        SectionFlags { bits: self.flags }
    }

    #[inline]
    pub const fn addr(&self) -> u64 {
        self.addr
    }

    #[inline]
    pub const fn file_offset(&self) -> usize {
        self.offset as _
    }

    #[inline]
    pub const fn size(&self) -> usize {
        self.size as _
    }

    #[inline]
    pub const fn entry_size(&self) -> u64 {
        self.entry_size
    }
}

impl fmt::Debug for SectionHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SectionHeader")
            .field("name_index", &format_args!("{:#x}", self.name_index))
            .field("section_type", &self.section_type())
            .field("flags", &format_args!("{:#x}", self.flags))
            .field("address", &format_args!("{:#018x}", self.addr))
            .field("offset", &format_args!("{:#x}", self.offset))
            .field("size", &format_args!("{:#x}", self.size))
            .field("link", &self.link)
            .field("info", &self.info)
            .field("address_align", &format_args!("{:#x}", self.addr_align))
            .field("entry_size", &format_args!("{:x}", self.entry_size))
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SectionType {
    Null,
    Progbits,
    Symtab,
    Strtab,
    Rela,
    Hash,
    Dynamic,
    Note,
    Nobits,
    Rel,
    Shlib,
    Dynsym,
    InitArray,
    FiniArray,
    PreinitArray,
    Group,
    SymtabShndx,
    EnvSpecific(u32),
    CpuSpecific(u32),
    UserSpecific(u32),
    Unknown(u32),
}

impl SectionType {
    pub const X86_64_UNWIND: Self = Self::CpuSpecific(0x70000001);
}

impl SectionType {
    pub const fn from_u32(x: u32) -> SectionType {
        match x {
            0 => SectionType::Null,
            1 => SectionType::Progbits,
            2 => SectionType::Symtab,
            3 => SectionType::Strtab,
            4 => SectionType::Rela,
            5 => SectionType::Hash,
            6 => SectionType::Dynamic,
            7 => SectionType::Note,
            8 => SectionType::Nobits,
            9 => SectionType::Rel,
            10 => SectionType::Shlib,
            11 => SectionType::Dynsym,
            14 => SectionType::InitArray,
            15 => SectionType::FiniArray,
            16 => SectionType::PreinitArray,
            17 => SectionType::Group,
            18 => SectionType::SymtabShndx,
            0x60000000..=0x6fffffff => SectionType::EnvSpecific(x),
            0x70000000..=0x7fffffff => SectionType::CpuSpecific(x),
            0x80000000..=0xffffffff => SectionType::UserSpecific(x),
            _ => SectionType::Unknown(x),
        }
    }
}

pub struct SectionFlags {
    pub(crate) bits: u64,
}

impl SectionFlags {
    #[inline]
    pub const fn write(self) -> bool {
        self.bits & 0x1 != 0
    }

    #[inline]
    pub const fn alloc(self) -> bool {
        self.bits & 0x2 != 0
    }

    #[inline]
    pub const fn execinstr(self) -> bool {
        self.bits & 0x4 != 0
    }

    #[inline]
    pub const fn merge(self) -> bool {
        self.bits & 0x10 != 0
    }

    #[inline]
    pub const fn strings(self) -> bool {
        self.bits & 0x20 != 0
    }

    #[inline]
    pub const fn info_link(self) -> bool {
        self.bits & 0x40 != 0
    }

    #[inline]
    pub const fn link_order(self) -> bool {
        self.bits & 0x80 != 0
    }

    #[inline]
    pub const fn os_nonconforming(self) -> bool {
        self.bits & 0x100 != 0
    }

    #[inline]
    pub const fn group(self) -> bool {
        self.bits & 0x200 != 0
    }

    #[inline]
    pub const fn tls(self) -> bool {
        self.bits & 0x400 != 0
    }
}
