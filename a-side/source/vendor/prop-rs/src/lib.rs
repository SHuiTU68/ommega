//! vendored 的上游属性实现（Magisk 那套）。
//!
//! CI 拿 `-D warnings` 跑整个 workspace，而这几条在上游的写法里是有意为之的，
//! 不为 lint 改它的形状（改了下一次同步上游还得再改回去）。
#![allow(
    clippy::too_many_arguments,
    clippy::unnecessary_cast,
    clippy::needless_range_loop
)]

mod persistent_prop;
mod prop_area;
mod prop_info;
pub mod property_context;

pub use persistent_prop::{
    check_proto, legacy_delete_prop, legacy_get_prop, legacy_list_props, legacy_set_prop,
    PersistentPropError, PersistentProperty, PersistentPropertyFile, PersistentResult,
    ANDROID_PERSISTENT_PROP_DIR, ANDROID_PERSISTENT_PROP_FILE,
};
pub use prop_area::{
    PropArea, PropAreaAllocationScan, PropAreaError, PropAreaHoleInfo, PropAreaObjectInfo,
    PropAreaObjectKind, PropWriteResult, Result,
};
pub use prop_info::{
    PropertyInfo, AREA_SERIAL_OFFSET, PROP_AREA_HEADER_SIZE, PROP_AREA_MAGIC, PROP_AREA_VERSION,
    PROP_INFO_SERIAL_OFFSET, PROP_NAME_MAX, PROP_VALUE_MAX,
};
pub use property_context::{ContextType, PropertyContext};
