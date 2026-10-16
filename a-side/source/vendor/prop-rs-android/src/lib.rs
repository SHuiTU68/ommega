//! vendored 的上游属性实现（Magisk 那套，Android 侧包装）。
//!
//! 同 `prop-rs`：CI 拿 `-D warnings` 跑整个 workspace，这几条在上游写法里是有意
//! 为之的，不为 lint 改它的形状。
#![allow(
    clippy::empty_line_after_doc_comments,
    clippy::io_other_error,
    clippy::needless_borrows_for_generic_args,
    clippy::ptr_arg
)]

pub mod mmap_prop_area;
pub mod persist;
pub mod resetprop;
pub mod sys_prop;
