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

use crate::{assert_struct_size, Elf, Section, StringTable, SHN_ABS, SHN_COMMON, SHN_UNDEF};
use core::fmt;

#[derive(Debug)]
pub struct SymbolTable<'elf> {
    elf: &'elf Elf<'elf>,
    _strtab: Option<StringTable<'elf>>,
    data: &'elf [crate::Sym],
}

impl<'elf> SymbolTable<'elf> {
    pub fn new(
        elf: &'elf Elf<'elf>,
        data: &'elf [Sym],
        strtab: Option<StringTable<'elf>>,
    ) -> SymbolTable<'elf> {
        Self {
            elf,
            _strtab: strtab,
            data,
        }
    }

    pub fn find<F>(&self, f: F) -> Option<Symbol<'elf>>
    where
        F: FnMut(&Symbol<'_>) -> bool,
    {
        self.data
            .iter()
            .map(|sym| Symbol { elf: self.elf, sym })
            .find(f)
    }
}

pub struct Symbol<'elf> {
    elf: &'elf Elf<'elf>,
    sym: &'elf Sym,
}

impl Symbol<'_> {
    pub fn name(&self) -> Option<&str> {
        self.elf.string_table()?.get_string(self.name_index())
    }

    pub fn section(&self) -> Option<Section<'_>> {
        self.elf.section(self.section_index)
    }
}

impl fmt::Debug for Symbol<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Symbol")
            .field("name", &self.name())
            .field("section", &self.section())
            .field("value", &self.value())
            .field("size", &self.size())
            .field("kind", &self.kind())
            .field("binding", &self.binding())
            .field("visibility", &self.visibility())
            .finish()
    }
}

impl core::ops::Deref for Symbol<'_> {
    type Target = Sym;

    fn deref(&self) -> &Self::Target {
        self.sym
    }
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct Sym {
    name_index: u32,
    info: SymInfo,
    section_index: u16,
    value: u64,
    size: u64,
}

assert_struct_size!(Sym, 24);

impl Sym {
    pub const fn name_index(&self) -> usize {
        self.name_index as usize
    }

    pub const fn binding(&self) -> Binding {
        self.info.binding()
    }

    pub const fn kind(&self) -> SymbolKind {
        self.info.kind()
    }

    pub const fn visibility(&self) -> Visibility {
        self.info.visibility()
    }

    pub const fn section_index(&self) -> u16 {
        self.section_index
    }

    pub const fn is_resolved(&self) -> bool {
        self.section_index == SHN_UNDEF
    }

    pub const fn is_absolute(&self) -> bool {
        self.section_index == SHN_ABS
    }

    pub const fn is_common(&self) -> bool {
        self.section_index == SHN_COMMON
    }

    /// Returns the symbol's value
    ///
    /// This value is interpreted slightly differently depending on the [type](FileType) of file:
    ///
    /// - Relocatable files:
    ///   An offset from the begging of the symbol's associated [`section`].
    ///   If the symbol refers to the common section (see [`is_common()`]), this value instead
    ///   holds alignment constraints for the symbol.
    ///
    /// - Executable and Shared Object files:
    ///   To make symbols more useful for the dynamic linker, this value contains a virtual
    ///   memory address instead of a section offset. The associated section is irrelevant in
    ///   this case.
    pub const fn value(&self) -> u64 {
        self.value
    }

    pub const fn as_ptr<T>(&self) -> *const T {
        self.value as _
    }

    pub const fn as_mut_ptr<T>(&self) -> *mut T {
        self.value as _
    }

    pub const fn size(&self) -> u64 {
        self.size
    }

    pub const fn contains_addr(&self, addr: u64) -> bool {
        self.value <= addr && addr < (self.value + self.size)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SymInfo {
    info: u8,
    other: u8,
}

impl SymInfo {
    pub const fn kind(self) -> SymbolKind {
        SymbolKind::from_sym_info(self)
    }

    pub const fn binding(self) -> Binding {
        Binding::from_sym_info(self)
    }

    pub const fn visibility(self) -> Visibility {
        Visibility::from_sym_info(self)
    }
}

/// Symbol Type
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SymbolKind {
    /// Unspecified type
    NoType,
    /// Data object
    Object,
    /// Function
    Func,
    /// Section
    Section,
    /// File
    File,
    /// Common block
    Common,
    /// Thread-Local Storage (TLS)
    Tls,
    /// Indirect function
    Ifunc,
    EnvSpecific(u8),
    CpuSpecific(u8),
    Unknown(u8),
}

impl SymbolKind {
    pub const fn from_sym_info(si: SymInfo) -> SymbolKind {
        match si.info & 0xf {
            0 => Self::NoType,
            1 => Self::Object,
            2 => Self::Func,
            3 => Self::Section,
            4 => Self::File,
            5 => Self::Common,
            6 => Self::Tls,
            10 => Self::Ifunc,
            x @ 10..=12 => Self::EnvSpecific(x),
            x @ 13..=15 => Self::CpuSpecific(x),
            x => Self::Unknown(x),
        }
    }
}

/// Symbol Binding
///
/// A symbol's binding determines its linkage visibility and behavior.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Binding {
    /// Local Symbol
    ///
    /// Local symbols are only visible within the file in which they are defined.
    /// Multiple local symbols of the same name may exist in different files.
    ///
    /// Local symbols take precedence over all other bindings.
    Local,
    /// Global Symbol
    ///
    /// Global symbols are visible to all object files. The definition of a global symbol
    /// in one file will satisfy an undefined reference in another file to a symbol of the
    /// same name.
    Global,
    /// Weak Symbols
    ///
    /// Weak symbols behave like [`Global`] symbols, but with a lower precedence.
    /// If a [`Global`] or COMMON symbol is defined with the same name, an error will not
    /// occur and the non-weak definition will be used.
    ///
    /// The link editor does *not* resolve undefined weak symbols from loaded archive libraries.
    Weak,
    EnvSpecific(u8),
    CpuSpecific(u8),
    Unknown(u8),
}

impl Binding {
    pub const fn from_sym_info(si: SymInfo) -> Binding {
        match si.info >> 4 {
            0 => Binding::Local,
            1 => Binding::Global,
            2 => Binding::Weak,
            x @ 10..=12 => Binding::EnvSpecific(x),
            x @ 13..=15 => Binding::CpuSpecific(x),
            x => Binding::Unknown(x),
        }
    }
}

/// Symbol Visibility
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Visibility {
    Default,
    Internal,
    Hidden,
    Protected,
}

impl Visibility {
    pub const fn from_sym_info(si: SymInfo) -> Visibility {
        match si.other & 0x3 {
            0 => Visibility::Default,
            1 => Visibility::Internal,
            2 => Visibility::Hidden,
            3 => Visibility::Protected,
            _ => unreachable!(),
        }
    }
}
