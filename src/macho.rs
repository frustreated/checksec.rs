//! Implements checksec for MachO binaries
#[cfg(feature = "color")]
use colored::*;
use goblin::mach::load_command::CommandVariant;
use goblin::mach::MachO;
use serde::{Deserialize, Serialize};
use std::fmt;

#[cfg(feature = "color")]
use crate::colorize_bool;
//use crate::shared::{Rpath, VecRpath};

const MH_ALLOW_STACK_EXECUTION: u32 = 0x0002_0000;
const MH_PIE: u32 = 0x0020_0000;
const MH_NO_HEAP_EXECUTION: u32 = 0x0100_0000;

/// Checksec result struct for MachO32/64 binaries
///
/// **Example**
///
/// ```rust
/// use checksec::macho::MachOCheckSecResults;
/// use goblin::mach::MachO;
/// use std::fs;
///
/// pub fn print_results(binary: &String) {
///     if let Ok(fp) = fs::File::open(&binary) {
///         if let Ok(buf) = fs::read(fp) {
///             if let Ok(macho) = MachO::parse(&buf) {
///                 println!("{:#?}", MachOCheckSecResults::parse(&macho));
///             }
///         }
///     }
/// }
/// ```
#[derive(Debug, Deserialize, Serialize)]
pub struct MachOCheckSecResults {
    /// Automatic Reference Counting
    pub arc: bool,
    /// Stack Canary
    pub canary: bool,
    /// Code Signature (codesign)
    pub code_signature: bool,
    /// Encrypted (LC_ENCRYPTION_INFO/LC_ENCRYPTION_INFO_64)
    pub encrypted: bool,
    /// Non-Executable Heap (MH_NO_HEAP_EXECUTION)
    pub nx_heap: bool,
    /// Non-Executable Stack (MH_ALLOW_STACK_EXECUTION)
    pub nx_stack: bool,
    /// Position Independent Executable (MH_PIE)
    pub pie: bool,
    /// Restrict segment
    pub restrict: bool,
    /// Load Command @rpath
    //rpath: VecRpath,
    pub rpath: bool,
}
impl MachOCheckSecResults {
    pub fn parse(macho: &MachO) -> MachOCheckSecResults {
        MachOCheckSecResults {
            arc: macho.has_arc(),
            canary: macho.has_canary(),
            code_signature: macho.has_code_signature(),
            encrypted: macho.has_encrypted(),
            nx_heap: macho.has_nx_heap(),
            nx_stack: macho.has_nx_stack(),
            pie: macho.has_pie(),
            restrict: macho.has_restrict(),
            rpath: macho.has_rpath(),
        }
    }
}
#[cfg(not(feature = "color"))]
impl fmt::Display for MachOCheckSecResults {
    /// Colorized human readable format output
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ARC: {} Canary: {} Code Signature: {} Encryption: {} \
            NX Heap: {} NX Stack: {} PIE: {} Restrict: {} \
            RPath: {}",
            self.arc,
            self.canary,
            self.code_signature,
            self.encrypted,
            self.nx_heap,
            self.nx_stack,
            self.pie,
            self.restrict,
            self.rpath
        )
    }
}

#[cfg(feature = "color")]
impl fmt::Display for MachOCheckSecResults {
    /// Colorized human readable format output
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
            "ARC:".bold(),
            colorize_bool!(self.arc),
            "Canary:".bold(),
            colorize_bool!(self.canary),
            "Code Signature:".bold(),
            colorize_bool!(self.code_signature),
            "Encrypted:".bold(),
            colorize_bool!(self.encrypted),
            "NX Heap:".bold(),
            colorize_bool!(self.nx_heap),
            "NX Stack:".bold(),
            colorize_bool!(self.nx_stack),
            "PIE:".bold(),
            colorize_bool!(self.pie),
            "Restrict:".bold(),
            colorize_bool!(self.restrict),
            "RPath:".bold(),
            //self.rpath
            colorize_bool!(self.rpath)
        )
    }
}

/// checksec Trait implementation for
/// [goblin::mach::MachO](https://docs.rs/goblin/latest/goblin/mach/struct.MachO.html)
///
/// **Example**
///
/// ```rust
/// use checksec::macho::MachOProperties;
/// use goblin::mach::MachO;
/// use std::fs;
///
/// pub fn print_results(binary: &String) {
///     if let Ok(fp) = fs::File::open(&binary) {
///         if let Ok(buf) = fs::read(fp) {
///             if let Ok(macho) = MachO::parse(&buf) {
///                 println!("arc: {}", macho.has_arc());
///             }
///         }
///     }
/// }
/// ```
pub trait MachOProperties {
    /// check import names for `_objc_release`
    fn has_arc(&self) -> bool;
    /// check import names for `___stack_chk_fail` or `___stack_chk_guard`
    fn has_canary(&self) -> bool;
    /// check data size of code signature in load commands
    fn has_code_signature(&self) -> bool;
    /// check if `cryptid` has a value set for EncryptionInfo32/64 in load
    /// commands
    fn has_encrypted(&self) -> bool;
    /// check `MH_NO_HEAP_EXECUTION` *(0x01000000)* in MachO header flags
    fn has_nx_heap(&self) -> bool;
    /// check `MH_ALLOW_STACK_EXECUTION` *(0x00020000)* in MachO header flags
    fn has_nx_stack(&self) -> bool;
    /// check `MH_PIE` *(0x00200000)* in MachO header flags
    fn has_pie(&self) -> bool;
    /// check for `___restrict` segment name
    fn has_restrict(&self) -> bool;
    //fn has_rpath(&self) -> VecRpath;
    /// check for RPath in load commands
    fn has_rpath(&self) -> bool;
}
impl MachOProperties for MachO<'_> {
    fn has_arc(&self) -> bool {
        if let Ok(imports) = self.imports() {
            for import in imports.iter() {
                if import.name == "_objc_release" {
                    return true;
                }
            }
        }
        false
    }
    fn has_canary(&self) -> bool {
        if let Ok(imports) = self.imports() {
            for import in imports.iter() {
                match import.name {
                    "___stack_chk_fail" => return true,
                    "___stack_chk_guard" => return true,
                    _ => continue,
                }
            }
        }
        false
    }
    fn has_code_signature(&self) -> bool {
        for loadcmd in self.load_commands.iter() {
            if let CommandVariant::CodeSignature(cmd) = loadcmd.command {
                // just check for existence, todo full validation
                if cmd.datasize > 0 {
                    return true;
                }
            }
        }
        false
    }
    fn has_encrypted(&self) -> bool {
        for loadcmd in self.load_commands.iter() {
            match loadcmd.command {
                CommandVariant::EncryptionInfo32(cmd) => {
                    if cmd.cryptid != 0 {
                        return true;
                    }
                }
                CommandVariant::EncryptionInfo64(cmd) => {
                    if cmd.cryptid != 0 {
                        return true;
                    }
                }
                _ => (),
            }
        }
        false
    }
    fn has_nx_heap(&self) -> bool {
        matches!(self.header.flags & MH_NO_HEAP_EXECUTION, x if x != 0)
    }
    fn has_nx_stack(&self) -> bool {
        !matches!(self.header.flags & MH_ALLOW_STACK_EXECUTION, x if x != 0)
    }
    fn has_pie(&self) -> bool {
        matches!(self.header.flags & MH_PIE, x if x != 0)
    }
    fn has_restrict(&self) -> bool {
        for segment in self.segments.iter() {
            if let Ok(name) = segment.name() {
                if name.to_string().to_lowercase() == "__restrict" {
                    return true;
                }
            }
        }
        false
    }
    //fn has_rpath(&self) -> VecRpath {
    fn has_rpath(&self) -> bool {
        // simply check for existence of @rpath command for now
        // parse out rpath entries similar to elf later
        // paths separated by `;` instead of `:` like the elf counterpart
        for loadcmd in self.load_commands.iter() {
            if let CommandVariant::Rpath(_) = loadcmd.command {
                return true;
                //return VecRpath::new(vec![Rpath::Yes("true".to_string())]);
            }
        }
        //VecRpath::new(vec![Rpath::None])
        false
    }
}
