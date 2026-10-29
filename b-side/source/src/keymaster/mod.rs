pub mod attest_proxy;
// HIDL keymaster@4.x 客户端：目前只挂着等验证，还没有任何调用方（`tee_ops` 仍只走 AIDL）。
// 等编码/回包两面的单测把形状钉死之后，再由 `relay_tee::get_real_keymint` 接进来。
#[allow(dead_code)]
pub mod hidl_km;
pub mod relay_tee;
pub mod session_db;
pub mod tee_ops;
