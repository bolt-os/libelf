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

use crate::assert_struct_size;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct RelocInfo(u64);

impl RelocInfo {
    #[inline]
    pub const fn new(symbol: u32, kind: RelocKind) -> RelocInfo {
        Self(((symbol as u64) << 32) | kind.0 as u64)
    }

    #[inline]
    pub const fn symbol(self) -> u32 {
        (self.0 >> 32) as u32
    }

    #[inline]
    pub const fn kind(self) -> RelocKind {
        RelocKind(self.0 as u32)
    }
}

#[repr(C)]
pub struct Rel {
    offset: u64,
    info: RelocInfo,
}

#[repr(C)]
#[derive(Clone, Default, Eq, Hash, PartialEq)]
pub struct Rela {
    pub offset: u64,
    pub info: RelocInfo,
    pub addend: i64,
}

assert_struct_size!(Rela, 24);

impl core::fmt::Debug for Rela {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Rela")
            .field("offset", &format_args!("{:#018x}", self.offset))
            .field("symbol", &format_args!("{:#018x}", self.sym()))
            .field("kind", &self.kind())
            .field("addend", &format_args!("{:#018x}", self.addend))
            .finish()
    }
}

impl Rela {
    #[inline]
    pub const fn sym(&self) -> u32 {
        self.info.symbol()
    }

    #[inline]
    pub const fn kind(&self) -> RelocKind {
        self.info.kind()
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd)]
pub struct RelocKind(u32);

macro_rules! reloc_kinds {
    ($(
        $(#[$meta:meta])*
        const $reloc_name:ident = $reloc_value:expr;
    )*) => {$(
        $(#[$meta])*
        pub const $reloc_name: Self = Self($reloc_value);
    )*};
}

/// RISC-V
///
/// - `A`   - the addend used to compute the value of the relocatable field
/// - `B`   - the base address at which the object has been loaded into memory
/// - `G`   - the offset into the GOT of the symbol
/// - `GOT` - the address of the Global Offset Table (GOT)
/// - `L`   - the place of the PLT entry for a symbol
/// - `P`   - the place of the storage unit being relocated
/// - `S`   - the value of the symbol referenced by the relocation entry
/// - `Z`   - the size of the symbol referenced by the relocation
impl RelocKind {
    reloc_kinds! {
        const RISCV_NONE                    = 0;
        const RISCV_32                      = 1;
        const RISCV_64                      = 2;
        const RISCV_RELATIVE                = 3;
        const RISCV_COPY                    = 4;
        const RISCV_JUMP_SLOT               = 5;
        const RISCV_TLS_DTPMOD32            = 6;
        const RISCV_TLS_DTPMOD64            = 7;
        const RISCV_TLS_DTPREL32            = 8;
        const RISCV_TLS_DTPREL64            = 9;
        const RISCV_TLS_TPREL32             = 10;
        const RISCV_TLS_TPREL64             = 11;
        const RISCV_BRANCH                  = 16;
        const RISCV_JAL                     = 17;
        const RISCV_CALL                    = 18;
        const RISCV_CALL_PLT                = 19;
        const RISCV_GOT_HI20                = 20;
        const RISCV_TLS_GOT_HI20            = 21;
        const RISCV_TLS_GD_HI20             = 22;
        const RISCV_PCREL_HI20              = 23;
        const RISCV_PCREL_LO12_I            = 24;
        const RISCV_PCREL_LO12_S            = 25;
        const RISCV_HI20                    = 26;
        const RISCV_LO12_I                  = 27;
        const RISCV_LO12_S                  = 28;
        const RISCV_TPREL_HI20              = 29;
        const RISCV_TPREL_LO12_I            = 30;
        const RISCV_TPREL_LO12_S            = 31;
        const RISCV_TPREL_ADD               = 32;
        const RISCV_ADD8                    = 33;
        const RISCV_ADD16                   = 34;
        const RISCV_ADD32                   = 35;
        const RISCV_ADD64                   = 36;
        const RISCV_SUB8                    = 37;
        const RISCV_SUB16                   = 38;
        const RISCV_SUB32                   = 39;
        const RISCV_SUB64                   = 40;
        const RISCV_GNU_VTINHERIT           = 41;
        const RISCV_GNU_VTENTRY             = 42;
        const RISCV_ALIGN                   = 43;
        const RISCV_RVC_BRANCH              = 44;
        const RISCV_RVC_JUMP                = 45;
        const RISCV_RVC_LUI                 = 46;
        const RISCV_RELAX                   = 51;
        const RISCV_SUB6                    = 52;
        const RISCV_SET6                    = 53;
        const RISCV_SET8                    = 54;
        const RISCV_SET16                   = 55;
        const RISCV_SET32                   = 56;
        const RISCV_32_PCREL                = 57;
        const RISCV_IRELATIVE               = 58;
    }
}

/// x86_64
///
/// - `A`   - the addend used to compute the value of the relocatable field
/// - `B`   - the base address at which the object has been loaded into memory
/// - `G`   - the offset into the GOT at which the relocation entry's symbol will reside
/// - `GOT` - the address of the Global Offset Table (GOT)
/// - `L`   - the place of the PLT entry for a symbol
/// - `P`   - the place of the storage unit being relocated
/// - `S`   - the value of the symbol referenced by the relocation entry
/// - `Z`   - the size of the symbol referenced by the relocation
impl RelocKind {
    reloc_kinds! {
        const X86_64_NONE            = 0;
        const X86_64_64              = 1;
        const X86_64_PC32            = 2;
        const X86_64_GOT32           = 3;
        const X86_64_PLT32           = 4;
        const X86_64_COPY            = 5;
        const X86_64_GLOB_DAT        = 6;
        const X86_64_JUMP_SLOT       = 7;
        const X86_64_RELATIVE        = 8;
        const X86_64_GOTPCREL        = 9;
        const X86_64_32              = 10;
        const X86_64_32S             = 11;
        const X86_64_16              = 12;
        const X86_64_PC16            = 13;
        const X86_64_8               = 14;
        const X86_64_PC8             = 15;
        const X86_64_DTPMOD64        = 16;
        const X86_64_DTPOFF64        = 17;
        const X86_64_TPOFF64         = 18;
        const X86_64_TLSGD           = 19;
        const X86_64_TLSLD           = 20;
        const X86_64_DTPOFF32        = 21;
        const X86_64_GOTTPOFF        = 22;
        const X86_64_TPOFF32         = 23;
        const X86_64_PC64            = 24;
        const X86_64_GOTOFF64        = 25;
        const X86_64_GOTPC32         = 26;
        const X86_64_SIZE32          = 32;
        const X86_64_SIZE64          = 33;
        const X86_64_GOTPC32_TLSDESC = 34;
        const X86_64_TLSDESC_CALL    = 35;
        const X86_64_TLSDESC         = 36;
        const X86_64_IRELATIVE       = 37;
        const X86_64_RELATIVE64      = 38;
        const X86_64_GOTPCRELX       = 41;
        const X86_64_REX_GOTPCRELX   = 42;
    }
}
