//! 顶 `android.hardware.security.keymint.IRemotelyProvisionedComponent/default`。
//!
//! 为什么需要：工程模式的 Key 状态页（`SecurityInterface.isRkpDefaultImported`）走
//! cryptoeng 的 61 号命令，内部按 IRPC **V3** 调 `generateCertificateRequestV2`。
//! 这台设备的原厂 RKP HAL 只报 `versionNumber = 1`（V1，没有 V2 方法），于是每次
//! 都回 `Status(-8, EX_SERVICE_SPECIFIC): '1: '`，工程模式那一行永远是红的。
//!
//! 这里用和 soterta 一样的做法：注册同名 binder 服务，`getHardwareInfo` 按 V3 报，
//! `generateCertificateRequestV2` 转给 daemon（daemon 再交给自己的 TA 出 CSR）。
//! 原厂 RKP 本来就用不了，接管不会比现状更差。

use rsbinder::hub;
use rsbinder::Binder;
use rsbinder::Interface;
use rsbinder::Parcel;
use rsbinder::Remotable;
use rsbinder::Result as BinderResult;
use rsbinder::Stability;
use rsbinder::TransactionCode;

pub const DESCRIPTOR: &str = "android.hardware.security.keymint.IRemotelyProvisionedComponent";
pub const SERVICE_NAME: &str =
    "android.hardware.security.keymint.IRemotelyProvisionedComponent/default";

// AIDL 的三个固定 transaction：INTERFACE_TRANSACTION 是 `"_NTF"`，
// getInterfaceVersion 是 0xFFFFFF，getInterfaceHash 是 0xFFFFFE。
// 客户端建完 proxy 会先问一遍这两个，回错会被当成服务不认识而退到别的路径。
const INTERFACE_TRANSACTION: TransactionCode = 0x5f4e_5446;
const GET_INTERFACE_VERSION: TransactionCode = 0x00ff_ffff;
const GET_INTERFACE_HASH: TransactionCode = 0x00ff_fffe;
const GET_HARDWARE_INFO: TransactionCode = 1;
const GENERATE_ECDSA_P256_KEY_PAIR: TransactionCode = 2;
const GENERATE_CERTIFICATE_REQUEST: TransactionCode = 3;
const GENERATE_CERTIFICATE_REQUEST_V2: TransactionCode = 4;

fn op_name(code: TransactionCode) -> &'static str {
    match code {
        GET_HARDWARE_INFO => "getHardwareInfo",
        GENERATE_ECDSA_P256_KEY_PAIR => "generateEcdsaP256KeyPair",
        GENERATE_CERTIFICATE_REQUEST => "generateCertificateRequest",
        GENERATE_CERTIFICATE_REQUEST_V2 => "generateCertificateRequestV2",
        _ => "unknown",
    }
}

struct RkpTa;

impl Remotable for RkpTa {
    fn descriptor() -> &'static str {
        DESCRIPTOR
    }

    fn on_transact(
        &self,
        code: TransactionCode,
        reader: &mut Parcel,
        reply: &mut Parcel,
    ) -> BinderResult<()> {
        if code == INTERFACE_TRANSACTION {
            reply.write(&DESCRIPTOR.to_string())?;
            return Ok(());
        }
        if code == GET_INTERFACE_VERSION {
            // RKP HAL 我们按 V3 报（generateCertificateRequestV2 就是 V3 引入的）。
            reply.write_i32(3)?;
            return Ok(());
        }
        if code == GET_INTERFACE_HASH {
            reply.write(&String::new())?;
            return Ok(());
        }
        log::info!("event=rkpta code={code} op={}", op_name(code));
        // AIDL 回复第一个 int 是 status。
        reply.write_i32(0)?;
        match code {
            GET_HARDWARE_INFO => write_hardware_info(reply)?,
            GENERATE_CERTIFICATE_REQUEST_V2 => {
                let keys = read_maced_keys(reader);
                let challenge = reader.read::<Vec<u8>>().unwrap_or_default();
                log::info!(
                    "event=rkpta generateCertificateRequestV2 keys={} challenge_len={}",
                    keys.len(),
                    challenge.len()
                );
                let csr = crate::ipc::with_ommega_retry(|client| {
                    client
                        .generateCertificateRequestV2(&keys, &challenge)
                        .map_err(|e| anyhow::anyhow!("{e:?}"))
                })
                .map_err(|e| {
                    log::warn!("event=rkpta csr failed: {e:#}");
                    rsbinder::StatusCode::UnknownTransaction
                })?;
                reply.write(&csr.to_vec())?;
            }
            // V1 的两个接口现在的工具都不会走到（它按 V3 调），给个明确失败。
            GENERATE_ECDSA_P256_KEY_PAIR | GENERATE_CERTIFICATE_REQUEST => {
                log::warn!("event=rkpta {op} unsupported", op = op_name(code));
                return Err(rsbinder::StatusCode::UnknownTransaction);
            }
            _ => {
                log::warn!("event=rkpta unknown_transaction code={code}");
                return Err(rsbinder::StatusCode::UnknownTransaction);
            }
        }
        Ok(())
    }

    fn on_dump(&self, _writer: &mut dyn std::io::Write, _args: &[String]) -> BinderResult<()> {
        Ok(())
    }
}

/// 读 `MacedPublicKey[]`：数组长度 + 每个 parcelable 的 `byte[] macedKey`。
fn read_maced_keys(
    reader: &mut Parcel,
) -> Vec<crate::android::hardware::security::keymint::MacedPublicKey::MacedPublicKey> {
    let count = reader.read::<i32>().unwrap_or(-1);
    if count <= 0 {
        return Vec::new();
    }
    let mut keys = Vec::with_capacity(count as usize);
    for _ in 0..count {
        match reader.read::<Vec<u8>>() {
            Ok(maced) => keys.push(
                crate::android::hardware::security::keymint::MacedPublicKey::MacedPublicKey {
                    macedKey: maced,
                },
            ),
            Err(_) => break,
        }
    }
    keys
}

/// 按 `RpcHardwareInfo` parcelable 的字段顺序写：versionNumber、rpcAuthorName、
/// supportedEekCurve、uniqueId（可空）、supportedNumKeysInCsr。
/// 按 AIDL parcelable 的 wire 格式写 `RpcHardwareInfo`。
///
/// 关键：parcelable 开头有一个 `int32 size`（后续字段的总字节数）。不写它，C++
/// 侧 `readFromParcel` 会把 `versionNumber` 当 size、把 `rpcAuthorName` 的头 4 字节
/// 当版本号（现场看到的就是 "On" 的 UTF-16 = 0x6E004F = 7209039）。
fn write_hardware_info(reply: &mut Parcel) -> BinderResult<()> {
    let info = crate::ipc::with_ommega_retry(|client| {
        client
            .getRpcHardwareInfo()
            .map_err(|e| anyhow::anyhow!("{e:?}"))
    })
    .map_err(|e| {
        log::warn!("event=rkpta hardware_info failed: {e:#}");
        rsbinder::StatusCode::UnknownTransaction
    })?;
    log::info!(
        "event=rkpta hardware_info version={} author={} eek={} unique={:?} keys={}",
        info.versionNumber,
        info.rpcAuthorName,
        info.supportedEekCurve,
        info.uniqueId,
        info.supportedNumKeysInCsr
    );

    let mut body = Vec::new();
    body.extend_from_slice(&info.versionNumber.to_le_bytes());
    push_string16(&mut body, &info.rpcAuthorName);
    body.extend_from_slice(&info.supportedEekCurve.to_le_bytes());
    match info.uniqueId.as_deref() {
        Some(unique) => push_string16(&mut body, unique),
        None => body.extend_from_slice(&(-1i32).to_le_bytes()),
    }
    body.extend_from_slice(&info.supportedNumKeysInCsr.to_le_bytes());
    // 补到 4 字节对齐（AIDL parcelable 总长度按 4 字节算），然后按 int32 写。
    // 不能用 write_u8 逐字节写：rsbinder 那边每次 write_u8 会补成 4 字节，
    // 整个 parcelable 会错位。
    while body.len() % 4 != 0 {
        body.push(0);
    }
    // AIDL parcelable 的 size 字段是「从 size 自己那个位置算到末尾」的总长，也就是
    // 要把这 4 字节算进去（AOSP 的 writeToParcel 是 end - start，start 就在 size 处）。
    // 只写字段长度会少 4，对端读完后 setDataPosition(start + size) 会提前 4 字节，
    // 于是判 BAD_VALUE。
    reply.write_i32(body.len() as i32 + 4)?;
    for chunk in body.chunks_exact(4) {
        reply.write_i32(i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))?;
    }
    Ok(())
}

/// String16（AOSP 的 `Parcel::writeString16` 形状）：int32 字符数 + UTF-16LE 码元
/// + 终止符 u16，然后整个 parcel 按 4 字节对齐。
///
/// 反汇编 `libbinder_ndk.so` 的 `AParcel_readString` 确认：它内部调的是
/// `Parcel::readString16Inplace`（读 `(len+1)*2` 字节并按 4 对齐），`@utf8InCpp`
/// 只影响返回给调用方的表示，不影响 wire 格式。
fn push_string16(out: &mut Vec<u8>, s: &str) {
    let utf16: Vec<u16> = s.encode_utf16().collect();
    out.extend_from_slice(&(utf16.len() as i32).to_le_bytes());
    for unit in &utf16 {
        out.extend_from_slice(&unit.to_le_bytes());
    }
    out.extend_from_slice(&0u16.to_le_bytes());
    while out.len() % 4 != 0 {
        out.push(0);
    }
}

pub fn serve(driver: &str) -> anyhow::Result<()> {
    rsbinder::ProcessState::init(driver, 15)
        .map_err(|error| anyhow::anyhow!("binder init on {driver} failed: {error}"))?;
    let binder = rsbinder::Binder::new_with_stability(RkpTa, Stability::Vintf);
    hub::add_service(SERVICE_NAME, binder.as_binder())?;
    log::info!("event=rkpta registered {SERVICE_NAME} on {driver}");
    rsbinder::ProcessState::join_thread_pool()?;
    Ok(())
}
