use der::asn1::OctetString;

/// AOSP `AttestationApplicationId`：
///
/// ```text
/// AttestationApplicationId ::= SEQUENCE {
///     packageInfos SEQUENCE OF AttestationPackageInfo,
///     signatureDigests SEQUENCE OF OCTET STRING
/// }
/// ```
///
/// 两个字段都是 SEQUENCE OF（tag 0x30）。以前这里写的是 `SetOfVec`（tag 0x31 SET），
/// 于是 AOSP 标准格式的 app_id 一律被拒：
/// `TagUnexpected { expected: Some(Tag(0x11: SET)), actual: Tag(0x10: SEQUENCE) }`。
/// 后果是 B 层的每一笔 attest 都在校验这关失败，请求被服务端摘到 keybox 层用 TEE
/// 身份铸链 —— 看起来就是「strongbox / 真机链路没生效」。
#[derive(der::Sequence, Debug)]
pub struct AttestationApplicationId {
    pub package_info_records: Vec<PackageInfoRecord>,
    pub signature_digests: Vec<OctetString>,
}

#[derive(der::Sequence, der::ValueOrd, Debug)]
pub struct PackageInfoRecord {
    pub package_name: OctetString,
    pub version: u64,
}
