/*
 * Copyright (C) 2022 The Android Open Source Project
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package android.hardware.security.keymint;

import android.hardware.security.keymint.MacedPublicKey;
import android.hardware.security.keymint.RpcHardwareInfo;

/**
 * HAL interface for remotely provisioned attestation keys.
 *
 * The AIDL is stable (VintfStability) because the framework and keystore2 talk to it
 * directly; a replacement implementation has to keep this exact shape.
 */
@VintfStability
interface IRemotelyProvisionedComponent {
    RpcHardwareInfo getHardwareInfo();
    byte[] generateEcdsaP256KeyPair(boolean testMode);
    MacedPublicKey[] generateCertificateRequest(
            in boolean testMode,
            in MacedPublicKey[] keysToSign,
            in byte[] endpointEncryptionCertChain,
            in byte[] challenge);
    byte[] generateCertificateRequestV2(in MacedPublicKey[] keysToSign, in byte[] challenge);
}
