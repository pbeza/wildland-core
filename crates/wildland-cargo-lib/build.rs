//
// Wildland Project
//
// Copyright © 2022 Golem Foundation
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3 as published by
// the Free Software Foundation.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

#[cfg(feature = "bindings")]
fn main() -> anyhow::Result<()> {
    use rusty_bind_build::parse_ffi_module;
    let ffi_code_target_dir = std::env::var("FFI_CODE_TARGET_DIR")
        .unwrap_or_else(|_| "./_generated_ffi_code/".to_owned());
    std::env::set_var("CARGO_CFG_TARGET_FEATURE", "");
    parse_ffi_module("src/ffi/mod.rs", &ffi_code_target_dir)?;
    println!("cargo:rerun-if-changed=src/ffi/mod.rs");
    Ok(())
}

#[cfg(not(feature = "bindings"))]
fn main() {}
