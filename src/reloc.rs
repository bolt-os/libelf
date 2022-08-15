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

#[cfg(target_arch = "x86_64")]
#[path = "arch/x86_64/reloc.rs"]
mod arch;
#[cfg(target_arch = "riscv64")]
#[path = "arch/riscv64/reloc.rs"]
mod arch;

use crate::assert_struct_size;
pub use arch::RelocKind;

#[repr(C)]
pub struct Rel {
    offset: u64,
    info: u64,
}

#[repr(C)]
#[derive(Clone, Default, Eq, Hash, PartialEq)]
pub struct Rela {
    pub offset: u64,
    pub info: u64,
    pub addend: i64,
}

assert_struct_size!(Rela, 24);

impl core::fmt::Debug for Rela {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Rela")
            .field("offset", &format_args!("{:#018x}", self.offset))
            .field(
                "symbol",
                &format_args!("{:#018x}", ((self.info >> 32) as u32)),
            )
            .field("kind", &self.kind())
            .field("addend", &format_args!("{:#018x}", self.addend))
            .finish()
    }
}

impl Rela {
    pub const fn sym(&self) -> u32 {
        (self.info >> 32) as _
    }

    pub const fn kind(&self) -> RelocKind {
        RelocKind::from_u32(self.info as _)
    }
}
