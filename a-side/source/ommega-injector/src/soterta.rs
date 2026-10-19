//! 软件 SOTER TA。
//!
//! 原厂 `vendor.qti.hardware.soter.ISoter/default` 只是 TEE 里那个 applet 的薄代理，
//! applet 起不来时整族调用一律 -20（PKG110 / 安卓 15 实测），宿主和应用一起哑，产线
//! 工具（工程模式那页）也跟着报红。这里把服务名原样接过来，用我们自己的软件实现应答，
//! 调用方（soterserver、oplus 的 cryptoeng）一行都不用改。
//!
//! 号码表跟宿主 dex 里 `ISoter$Stub$Proxy` 对过：1/3/4/5/7/8/9/10/11/12/13 是运行时
//! 那 11 个方法；2/6/14 原厂 HAL 是空号，只有产线 provisioning（cryptoeng）会调，
//! 我们用同一套 ASK 顶上，工程模式的 SOTER key 就看 14 号那一笔是不是 0。

use rsbinder::{
    hub, Binder, Interface, Parcel, Remotable, Result as BinderResult, Stability, TransactionCode,
};

use crate::hook::soter;
use crate::hook::soter_local::{self, Answer};

pub const SERVICE_NAME: &str = "vendor.qti.hardware.soter.ISoter/default";
const DESCRIPTOR: &str = "vendor.qti.hardware.soter.ISoter";

/// 原厂 HAL 声明的 14 个号码。
const EXPORT_ASK_PUBLIC_KEY: u32 = 1;
const EXPORT_ATTK_PUBLIC_KEY: u32 = 2;
const EXPORT_AUTH_KEY_PUBLIC_KEY: u32 = 3;
const FINISH_SIGN: u32 = 4;
const GENERATE_ASK_KEY_PAIR: u32 = 5;
const GENERATE_ATTK_KEY_PAIR: u32 = 6;
const GENERATE_AUTH_KEY_PAIR: u32 = 7;
const GET_DEVICE_ID: u32 = 8;
const HAS_ASK_ALREADY: u32 = 9;
const HAS_AUTH_KEY: u32 = 10;
const INIT_SIGN: u32 = 11;
const REMOVE_ALL_UID_KEY: u32 = 12;
const REMOVE_AUTH_KEY: u32 = 13;
const VERIFY_ATTK_KEY_PAIR: u32 = 14;

/// servicemanager 注册时会用这个 code 来问“你这个 binder 是什么接口”，拿不到名字就
/// 记成空列表，之后 vendor 客户端按（名字 + 接口名）取服务一律 null。
const INTERFACE_TRANSACTION: u32 = 0x5f4e_5446;

fn op_name(code: u32) -> &'static str {
    match code {
        EXPORT_ASK_PUBLIC_KEY => "exportAskPublicKey",
        EXPORT_ATTK_PUBLIC_KEY => "exportAttkPublicKey",
        EXPORT_AUTH_KEY_PUBLIC_KEY => "exportAuthKeyPublicKey",
        FINISH_SIGN => "finishSign",
        GENERATE_ASK_KEY_PAIR => "generateAskKeyPair",
        GENERATE_ATTK_KEY_PAIR => "generateAttkKeyPair",
        GENERATE_AUTH_KEY_PAIR => "generateAuthKeyPair",
        GET_DEVICE_ID => "getDeviceId",
        HAS_ASK_ALREADY => "hasAskAlready",
        HAS_AUTH_KEY => "hasAuthKey",
        INIT_SIGN => "initSign",
        REMOVE_ALL_UID_KEY => "removeAllUidKey",
        REMOVE_AUTH_KEY => "removeAuthKey",
        VERIFY_ATTK_KEY_PAIR => "verifyAttkKeyPair",
        _ => "unknown",
    }
}

/// `int xxx(out byte[] data)` 那一类：返回值 + AIDL 原生 byte[]。
/// 不是 `SoterBufferReturn` parcelable —— cryptoeng（C++ ndk 客户端）的反汇编里就是
/// 一个 vector<uint8_t> out 加一个 int 返回，按 parcelable 拼它会在解析时越界崩。
fn write_bytes_reply(reply: &mut Parcel, data: &[u8], code: i32) -> BinderResult<()> {
    reply.write_i32(code)?;
    reply.write(&data.to_vec())?;
    Ok(())
}

/// `SoterInitReturn`：`[非空标记][总长=16][status][session]`，两家 AIDL 一样。
/// 注意这个方法是**返回 parcelable 本身**，没有“方法返回值”那一格。
fn write_init_return(reply: &mut Parcel, status: i32, session: i64) -> BinderResult<()> {
    reply.write_i32(1)?;
    reply.write_i32(16)?;
    reply.write_i32(status)?;
    reply.write_i64(session)?;
    Ok(())
}

struct SoterTa;

impl Remotable for SoterTa {
    fn descriptor() -> &'static str {
        DESCRIPTOR
    }

    fn on_transact(
        &self,
        code: TransactionCode,
        reader: &mut Parcel,
        reply: &mut Parcel,
    ) -> BinderResult<()> {
        log::info!("event=soterta code={code} op={}", op_name(code));
        if code == INTERFACE_TRANSACTION {
            reply.write(&DESCRIPTOR.to_string())?;
            return Ok(());
        }
        // 先按 AIDL 定义把入参读干净：读错位的话后面拿到的 uid/alias 全是垃圾。
        let mut uid = None;
        let mut alias = None;
        let mut challenge = None;
        let mut session = None;
        match code {
            EXPORT_ASK_PUBLIC_KEY
            | GENERATE_ASK_KEY_PAIR
            | HAS_ASK_ALREADY
            | REMOVE_ALL_UID_KEY
            | EXPORT_ATTK_PUBLIC_KEY
            | GENERATE_ATTK_KEY_PAIR => {
                uid = reader.read::<i32>().ok();
            }
            EXPORT_AUTH_KEY_PUBLIC_KEY
            | GENERATE_AUTH_KEY_PAIR
            | HAS_AUTH_KEY
            | REMOVE_AUTH_KEY => {
                uid = reader.read::<i32>().ok();
                alias = reader.read::<String>().ok();
            }
            INIT_SIGN => {
                uid = reader.read::<i32>().ok();
                alias = reader.read::<String>().ok();
                challenge = reader.read::<String>().ok();
            }
            FINISH_SIGN => {
                session = reader.read::<i64>().ok();
            }
            _ => {}
        }

        // 产线 provisioning 那三个（原厂 HAL 是空号）：工程模式的 SOTER key 就看 14 号。
        match code {
            VERIFY_ATTK_KEY_PAIR => {
                reply.write_i32(0)?;
                return Ok(());
            }
            GENERATE_ATTK_KEY_PAIR => {
                reply.write_i32(0)?;
                return Ok(());
            }
            EXPORT_ATTK_PUBLIC_KEY => {
                // ATTK 用我们自己的 ASK 顶：软件 TA 里 ASK 兼 ATTK，公钥就是那一把。
                let document = soter_local::ask_public_document(uid.unwrap_or(-1));
                write_bytes_reply(reply, &document, 0)?;
                return Ok(());
            }
            _ => {}
        }

        // 剩下的 11 个号码走本地软件 TA：ASK/AuthKey/会话全部复用注入侧那套实现，
        // 这样接管和注入两条路签出来的东西是同一种，不会出现两种假链。
        let call = soter::SoterCall {
            hal: true,
            hidl: false,
            has_return_code: true,
            code,
            wire_code: code,
            op: op_name(code),
            uid,
            alias: alias.clone(),
            challenge: challenge.clone(),
            session,
            key: None,
            caller_uid: None,
            data_size: 0,
        };
        match soter_local::answer(&call) {
            Some(Answer::Code(code)) => {
                reply.write_i32(code)?;
            }
            Some(Answer::Buffer { code, data }) => {
                write_bytes_reply(reply, data.as_deref().unwrap_or(&[]), code)?;
            }
            Some(Answer::Init { status, session }) => {
                // initSign 返回的是 parcelable 本身，没有「方法返回值」那一格。
                write_init_return(reply, status, session)?;
            }
            None => {
                // 没有真身份可报、或参数不够：回一个明确的错误码，别回 0 让调用方
                // 拿着空数据继续跑。NOT_FOUND 跟真机一样。
                if matches!(
                    code,
                    EXPORT_ASK_PUBLIC_KEY | EXPORT_AUTH_KEY_PUBLIC_KEY | FINISH_SIGN
                ) {
                    write_bytes_reply(reply, &[], -4)?;
                } else {
                    reply.write_i32(-4)?;
                }
            }
        }
        Ok(())
    }

    fn on_dump(&self, _writer: &mut dyn std::io::Write, _args: &[String]) -> BinderResult<()> {
        Ok(())
    }
}

/// 起服务并常驻。返回即表示服务名已经抢到手。
///
/// `driver` 决定注册到哪个 servicemanager：vendor 的 AIDL HAL 习惯落在
/// `/dev/vndbinder`（vndservicemanager），应用侧的 Java 客户端只看 `/dev/binder`。
pub fn serve(driver: &str) -> anyhow::Result<()> {
    rsbinder::ProcessState::init(driver, 0)
        .map_err(|error| anyhow::anyhow!("binder init on {driver} failed: {error}"))?;
    let binder = Binder::new_with_stability(SoterTa, Stability::Vintf);
    hub::add_service(SERVICE_NAME, binder.as_binder())?;
    log::info!("event=soterta registered {SERVICE_NAME} on {driver}");
    // 注册完就得有人处理事务：主线程进 binder 事件循环（rsbinder 的 join_thread_pool）
    rsbinder::ProcessState::join_thread_pool()?;
    Ok(())
}
