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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Version {
    None,
    Current,
    Unknown(u32),
}

impl Version {
    pub const fn from_u32(x: u32) -> Version {
        match x {
            0 => Version::None,
            1 => Version::Current,
            _ => Version::Unknown(x),
        }
    }

    pub const fn from_u8(x: u8) -> Version {
        Self::from_u32(x as u32)
    }
}

/// ELF File Type
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ElfType {
    /// No file type
    None,
    /// Relocatable object file
    Rel,
    /// Executable file
    Exec,
    /// Shared object file
    Dyn,
    /// Core file
    Core,
    /// Environment-specific
    EnvSpecific(u16),
    /// Processor-specific
    CpuSpecific(u16),
    Unknown(u16),
}

impl ElfType {
    pub const fn from_u16(x: u16) -> ElfType {
        match x {
            0 => ElfType::None,
            1 => ElfType::Rel,
            2 => ElfType::Exec,
            3 => ElfType::Dyn,
            4 => ElfType::Core,
            0xfe00..=0xfeff => ElfType::EnvSpecific(x),
            0xff00..=0xffff => ElfType::CpuSpecific(x),
            _ => ElfType::Unknown(x),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Machine {
    None,
    X86_64,
    Aarch64,
    Riscv,
    Unknown(u16),
}

impl Machine {
    pub const fn from_u16(x: u16) -> Machine {
        match x {
            0 => Machine::None,
            62 => Machine::X86_64,
            183 => Machine::Aarch64,
            243 => Machine::Riscv,
            _ => Machine::Unknown(x),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Class {
    None,
    Bits32,
    Bits64,
    Unknown(u8),
}

impl Class {
    pub const fn from_u8(x: u8) -> Class {
        match x {
            0 => Self::None,
            1 => Self::Bits32,
            2 => Self::Bits64,
            _ => Self::Unknown(x),
        }
    }

    pub const fn to_u8(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Bits32 => 1,
            Self::Bits64 => 2,
            Self::Unknown(x) => x,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Data {
    None,
    TwosCompLittle,
    TwosCompBig,
    Unknown(u8),
}

impl Data {
    pub const fn from_u8(x: u8) -> Data {
        match x {
            0 => Self::None,
            1 => Self::TwosCompLittle,
            2 => Self::TwosCompBig,
            _ => Self::Unknown(x),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum OsAbi {
    SysV,
    NetBSD,
    Linux,
    FreeBSD,
    OpenBSD,
    Standalone,
    Unknown(u8),
}

impl OsAbi {
    pub const fn from_u8(x: u8) -> OsAbi {
        match x {
            0 => Self::SysV,
            2 => Self::NetBSD,
            3 => Self::Linux,
            9 => Self::FreeBSD,
            12 => Self::OpenBSD,
            255 => Self::Standalone,
            _ => Self::Unknown(x),
        }
    }
}
