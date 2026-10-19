package top.jiyin004.ommega;

import android.hardware.security.keymint.SecurityLevel;
import android.hardware.security.keymint.Certificate;
import android.hardware.security.keymint.Tag;
import android.hardware.security.keymint.MacedPublicKey;
import android.hardware.security.keymint.RpcHardwareInfo;
import android.system.keystore2.Domain;
import android.system.keystore2.IKeystoreSecurityLevel;
import android.system.keystore2.KeyDescriptor;
import android.system.keystore2.KeyEntryResponse;

import top.jiyin004.ommega.ISecurityLevel;
import top.jiyin004.ommega.CallerInfo;

interface IKeymintService {

    IKeystoreSecurityLevel getSecurityLevel(in SecurityLevel securityLevel);

    ISecurityLevel getCustomSecurityLevel(in SecurityLevel securityLevel);

    KeyEntryResponse getKeyEntry(in @nullable CallerInfo ctx, in KeyDescriptor key);


    void updateSubcomponent(in @nullable CallerInfo ctx, in KeyDescriptor key,
                            in @nullable byte[] publicCert, in @nullable byte[] certificateChain);


    KeyDescriptor[] listEntries(in @nullable CallerInfo ctx, in Domain domain, in long nspace);


    void deleteKey(in @nullable CallerInfo ctx, in KeyDescriptor key);


    KeyDescriptor grant(in @nullable CallerInfo ctx, in KeyDescriptor key, in int granteeUid, in int accessVector);


    void ungrant(in @nullable CallerInfo ctx, in KeyDescriptor key, in int granteeUid);


    int getNumberOfEntries(in @nullable CallerInfo ctx, in Domain domain, in long nspace);


    KeyDescriptor[] listEntriesBatched(in @nullable CallerInfo ctx, in Domain domain, in long nspace,
            in @nullable String startingPastAlias);

    byte[] getSupplementaryAttestationInfo(in Tag tag);

    void updateEcKeybox(in @nullable CallerInfo ctx, in byte[] key, in List<Certificate> chain);

    void updateRsaKeybox(in @nullable CallerInfo ctx, in byte[] key, in List<Certificate> chain);

    boolean isGrant(in @nullable CallerInfo ctx, in KeyDescriptor grant);

    // RKP（IRemotelyProvisionedComponent）透传：工程模式的 Key 状态页会按 IRPC V3
    // 调 generateCertificateRequestV2，而原厂 HAL 只报 V1、没这个方法。这条 RPC 把
    // TA 自己的 RKP 能力交给 injector 那边顶上去的 HAL 服务用。
    RpcHardwareInfo getRpcHardwareInfo();

    byte[] generateCertificateRequestV2(in MacedPublicKey[] keysToSign, in byte[] challenge);
}
