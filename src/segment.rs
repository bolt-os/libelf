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
use core::fmt;

pub struct Segment<'elf> {
    elf: &'elf Elf<'elf>,
    hdr: &'elf ProgramHeader,
}

impl<'elf> Segment<'elf> {
    pub(crate) fn new(elf: &'elf Elf<'elf>, hdr: &'elf ProgramHeader) -> Segment<'elf> {
        Self { elf, hdr }
    }
}

impl Segment<'_> {
    pub fn file_data(&self) -> &[u8] {
        &self.elf.data[self.file_offset()..][..self.file_size()]
    }
}

impl fmt::Debug for Segment<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Segment")
            .field("kind", &self.kind())
            .field("flags", &self.flags())
            .field("virt_address", &format_args!("{:#018x}", self.vaddr))
            .field("phys_address", &format_args!("{:#018x}", self.paddr))
            .field("file_offset", &format_args!("{:#x}", self.file_offset))
            .field("file_size", &format_args!("{:#x}", self.file_size))
            .field("mem_size", &format_args!("{:#x}", self.mem_size))
            .field("alignment", &format_args!("{:#x}", self.alignment))
            .finish()
    }
}

impl core::ops::Deref for Segment<'_> {
    type Target = ProgramHeader;

    fn deref(&self) -> &Self::Target {
        self.hdr
    }
}

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct SegmentFlags : u32 {
        const EXEC  = 0x1;
        const WRITE = 0x2;
        const READ  = 0x4;
    }
}

#[repr(C)]
pub struct ProgramHeader {
    kind: u32,
    flags: SegmentFlags,
    file_offset: u64,
    vaddr: u64,
    paddr: u64,
    file_size: u64,
    mem_size: u64,
    alignment: u64,
}

assert_struct_size!(ProgramHeader, 56);

impl ProgramHeader {
    pub const fn kind(&self) -> SegmentKind {
        SegmentKind::from_u32(self.kind)
    }

    pub const fn flags(&self) -> SegmentFlags {
        self.flags
    }

    pub const fn file_offset(&self) -> usize {
        self.file_offset as _
    }

    pub const fn file_size(&self) -> usize {
        self.file_size as _
    }

    pub const fn mem_size(&self) -> usize {
        self.mem_size as _
    }

    pub const fn virtual_address(&self) -> u64 {
        self.vaddr
    }

    pub const fn physical_address(&self) -> u64 {
        self.paddr
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SegmentKind {
    Null,
    Load,
    Dynamic,
    Interp,
    Note,
    Shlib,
    Phdr,
    Tls,
    Unwind,
    EhFrame,
    Stack,
    Relro,
    EnvSpecific(u32),
    CpuSpecific(u32),
    Unknown(u32),
}

impl SegmentKind {
    pub const fn from_u32(x: u32) -> SegmentKind {
        match x {
            0 => Self::Null,
            1 => Self::Load,
            2 => Self::Dynamic,
            3 => Self::Interp,
            4 => Self::Note,
            5 => Self::Shlib,
            6 => Self::Phdr,
            7 => Self::Tls,
            0x6464e550 => Self::Unwind,
            0x6474e550 => Self::EhFrame,
            0x6474e551 => Self::Stack,
            0x6474e552 => Self::Relro,
            0x60000000..=0x6fffffff => Self::EnvSpecific(x),
            0x70000000..=0x7fffffff => Self::CpuSpecific(x),
            _ => Self::Unknown(x),
        }
    }
}
