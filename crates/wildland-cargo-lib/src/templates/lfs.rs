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

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use wildland_corex::{
    StorageTemplate, StorageTemplateError, CONTAINER_NAME_PARAM, CONTAINER_UUID_PARAM,
};

use crate::api::storage::StorageBackendType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalFilesystemStorageTemplate {
    local_dir: PathBuf,
    container_prefix: String,
}

impl LocalFilesystemStorageTemplate {
    pub fn new(local_dir: impl Into<PathBuf>) -> Self {
        Self {
            local_dir: local_dir.into(),
            container_prefix: format!(
                "{{{{ {CONTAINER_NAME_PARAM} }}}}/{{{{ {CONTAINER_UUID_PARAM} }}}}"
            ),
        }
    }
}

impl TryFrom<LocalFilesystemStorageTemplate> for StorageTemplate {
    type Error = StorageTemplateError;
    fn try_from(lfst: LocalFilesystemStorageTemplate) -> Result<Self, Self::Error> {
        StorageTemplate::try_new(StorageBackendType::LocalFilesystem, lfst)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn test_serializing_lfs_template() {
        let template: StorageTemplate = LocalFilesystemStorageTemplate::new("/home/user/wildland")
            .try_into()
            .unwrap();
        let template_uuid = template.uuid();

        let expected_json_form = json!({
            "uuid": template_uuid,
            "backend_type": "LocalFilesystem",
            "name": null,
            "template": {
                "local_dir": "/home/user/wildland",
                "container_prefix": "{{ CONTAINER_NAME }}/{{ CONTAINER_UUID }}"
            }
        });

        assert_eq!(expected_json_form, serde_json::to_value(&template).unwrap());
    }
}
