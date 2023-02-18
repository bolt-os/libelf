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
use core::mem::size_of;

pub struct DynamicTable<'a, 'elf> {
    _elf: &'a Elf<'elf>,
    data: &'elf [Dyn],
}

impl<'a, 'elf> DynamicTable<'a, 'elf> {
    pub fn new(_elf: &'a Elf<'elf>, data: &'elf [u8]) -> DynamicTable<'a, 'elf> {
        let len = data.len() / size_of::<Dyn>();
        let data = data.as_ptr().cast::<Dyn>();
        let mut dyntab = unsafe { core::slice::from_raw_parts(data, len) };

        if !dyntab.is_empty() {
            let last = dyntab.len() - 1;
            assert_eq!(dyntab[last].tag, 0);
            dyntab = &dyntab[..last];
        }

        Self { _elf, data: dyntab }
    }
}

impl<'elf> DynamicTable<'_, 'elf> {
    #[inline]
    pub fn table_raw(&self) -> &'elf [Dyn] {
        self.data
    }
}

#[repr(C)]
#[derive(Clone, Copy, Eq, Hash, PartialEq, PartialOrd)]
pub struct Dyn {
    tag: isize,
    value: usize,
}

assert_struct_size!(Dyn, 16);

impl Dyn {
    #[inline]
    pub fn tag(&self) -> DynTag {
        DynTag::from_raw(self.tag)
    }

    #[inline(always)]
    pub fn value(&self) -> usize {
        self.value
    }

    #[inline]
    pub fn as_ptr<T>(&self) -> *const T {
        self.value as *const T
    }

    #[inline]
    pub fn as_mut_ptr<T>(&self) -> *mut T {
        self.value as *mut T
    }
}

impl core::fmt::Debug for Dyn {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Dyn")
            .field("tag", &self.tag())
            .field("value", &format_args!("{:#018x}", self.value))
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DynTag {
    Null,
    Needed,
    PltRelSize,
    PltGot,
    Hash,
    /// Address of the dynamic string table
    Strtab,
    /// Address of the dynamic symbol table
    Symtab,
    /// Address of [`Rela`](crate::reloc::Rela) type relocation table
    Rela,
    /// Size, in bytes, of the [`Rela`](DynTag::Rela) relocation table
    RelaSize,
    /// Size, in bytes, of each entry in the [`Rela`](DynTag::Rela) relocation table
    RelaEnt,
    /// Size, in bytes, of the dynamic string table
    StrtabSize,
    SymEnt,
    Init,
    Fini,
    Soname,
    Rpath,
    Symbolic,
    Rel,
    RelSize,
    RelEnt,
    PltRel,
    Debug,
    TextRel,
    JmpRel,
    BindNow,
    /// Pointer to an array of pointers to initialization functions
    InitArray,
    /// Pointer to an array of pointers to termination functions
    FiniArray,
    /// Size, in bytes, of the array of initialization function pointers ([`InitArray`])
    ///
    /// [`InitArray`]: DynTag::InitArray
    InitArraySize,
    /// Size, in bytes, of the array of termination function pointers ([`FiniArray`])
    ///
    /// [`FiniArray`]: DynTag::FiniArray
    FiniArraySize,
    Runpath,
    Flags,
    Encoding,
    PreinitArray,
    PreinitArraySize,
    GnuHash,
    RelaCount,
    Flags1,
    EnvSpecific(isize),
    CpuSpecific(isize),
    Unknown(isize),
}

impl DynTag {
    #[inline]
    pub const fn from_raw(tag: isize) -> Self {
        use DynTag::*;

        match tag {
            0 => Null,
            1 => Needed,
            2 => PltRelSize,
            3 => PltGot,
            4 => Hash,
            5 => Strtab,
            6 => Symtab,
            7 => Rela,
            8 => RelaSize,
            9 => RelaEnt,
            10 => StrtabSize,
            11 => SymEnt,
            12 => Init,
            13 => Fini,
            14 => Soname,
            15 => Rpath,
            16 => Symbolic,
            17 => Rel,
            18 => RelSize,
            19 => RelEnt,
            20 => PltRel,
            21 => Debug,
            22 => TextRel,
            23 => JmpRel,
            24 => BindNow,
            25 => InitArray,
            26 => FiniArray,
            27 => InitArraySize,
            28 => FiniArraySize,
            29 => Runpath,
            30 => Flags,
            32 => PreinitArray,
            33 => PreinitArraySize,
            0x6ffffef5 => GnuHash,
            0x6ffffff9 => RelaCount,
            0x6ffffffb => Flags1,
            0x60000000..=0x6fffffff => EnvSpecific(tag),
            0x70000000..=0x7fffffff => CpuSpecific(tag),
            _ => Unknown(tag),
        }
    }
}

impl From<isize> for DynTag {
    fn from(tag: isize) -> Self {
        Self::from_raw(tag)
    }
}
