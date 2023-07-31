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
            assert_eq!(dyntab[last].tag, DynTag::NULL);
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
    pub tag: DynTag,
    pub value: usize,
}

assert_struct_size!(Dyn, 16);

impl Dyn {
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
            .field("tag", &self.tag)
            .field("value", &format_args!("{:#018x}", self.value))
            .finish()
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DynTag(isize);

macro_rules! dyn_tags {
    ($($(#[$meta:meta])* const $name:ident = $val:expr;)*) => {
        impl DynTag {
            $($(#[$meta])*pub const $name: DynTag = Self($val);)*
        }

        impl core::fmt::Debug for DynTag {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                #[allow(unreachable_patterns)]
                let name = match self.0 {
                    $($val => stringify!($name),)*
                    x => return write!(f, "DynTag({x:#x})"),
                };
                write!(f, "DynTag::{name}")
            }
        }
    };
}

dyn_tags! {
    const NULL              = 0;
    const NEEDED            = 1;
    const PLTRELSZ          = 2;
    const PLTGOT            = 3;
    const HASH              = 4;
    const STRTAB            = 5;
    const SYMTAB            = 6;
    const RELA              = 7;
    const RELASZ            = 8;
    const RELAENT           = 9;
    const STRSZ             = 10;
    const SYMENT            = 11;
    const INIT              = 12;
    const FINI              = 13;
    const SONAME            = 14;
    const RPATH             = 15;
    const SYMBOLIC          = 16;
    const REL               = 17;
    const RELSZ             = 18;
    const RELENT            = 19;
    const PLTREL            = 20;
    const DEBUG             = 21;
    const TEXTREL           = 22;
    const JMPREL            = 23;
    const BIND_NOW          = 24;
    const INIT_ARRAY        = 25;
    const FINI_ARRAY        = 26;
    const INIT_ARRAYSZ      = 27;
    const FINI_ARRAYSZ      = 28;
    const RUNPATH           = 29;
    const FLAGS             = 30;
    const PREINIT_ARRAY     = 32;
    // This needs to come second so `PREINIT_ARRAY` is chosed for Debug output
    const ENCODING          = 32;
    const PREINIT_ARRAYSZ   = 33;
    const GNU_HASH          = 0x6ffffef5;
    const RELACOUNT         = 0x6ffffff9;
    const RELCOUNT          = 0x6ffffffa;
    const FLAGS_1           = 0x6ffffffb;

    const LOOS          = 0x60000000;
    const HIOS          = 0x6FFFFFFF;
    const LOPROC        = 0x70000000;
    const HIPROC        = 0x7FFFFFFF;
}
