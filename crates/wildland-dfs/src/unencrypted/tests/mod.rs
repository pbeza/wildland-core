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

// Tests are divided into modules according to the used PathTranslator, which is a trait specifying the logic of translating
// absolute paths in terms of a user's namespace into exposed paths in a filesystem frontend. This logic handles such
// problems as conflicting paths in the user's namespace from different data sources.
mod events;
mod fs_stubs;
mod uuid_dir_path_translator;

pub use fs_stubs::mufs::*;
pub use fs_stubs::unresponsive_fs::*;
pub use fs_stubs::*;
