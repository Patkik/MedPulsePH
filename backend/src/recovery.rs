//! Coordinated split-vault backup/restore reconciliation (`I11`,
//! `25-availability-and-disaster-recovery.md`, `.agents/skills/firebase-deploy-scale/SKILL.md`).
//!
//! Enforces:
//! - Restoring an older Vault A database snapshot must NEVER roll back an NFC wristband's
//!   `high_water_counter` below newer protected audit/recovery evidence, resurrect a
//!   `Frozen` or `Retired` wristband, or allow pre-restore `VerifiedClinicSession` tokens
//!   to remain valid (`25-availability-and-disaster-recovery.md`).
//! - If protected recovery evidence for a band is missing or inconsistent after a restore,
//!   the band fails closed into `BandStatus::Frozen` until verified or replaced.

use crate::{
    auth::BandStatus,
    crypto::BlindPatientId,
    nfc::{DurableBandReplayState, Ntag424TagKeys},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Snapshot of a patient's band and session revocation state restored from a Vault A backup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VaultABandBackupSnapshot {
    pub blind_patient_id: BlindPatientId,
    pub tag_uid: [u8; 7],
    pub snapshot_band_status: BandStatus,
    pub snapshot_high_water_counter: u32,
    pub snapshot_revocation_version: u64,
    pub backup_captured_at_unix: u64,
}

/// Independently protected recovery checkpoint / audit evidence captured after the backup snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtectedRecoveryEvidence {
    pub blind_patient_id: BlindPatientId,
    pub tag_uid: [u8; 7],
    pub observed_high_water_counter: u32,
    pub latest_known_band_status: BandStatus,
    pub observed_revocation_version: u64,
    pub evidence_verified: bool,
}

/// Result of reconciling a restored Vault A backup snapshot with protected recovery evidence.
pub struct ReconciledBandRecovery {
    pub restored_band_state: DurableBandReplayState,
    pub post_restore_revocation_version: u64,
    pub quarantined_for_manual_verification: bool,
}

/// Typed errors for coordinated backup/restore reconciliation.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum RecoveryError {
    #[error("backup snapshot and recovery evidence refer to different patients or tag UIDs")]
    MismatchedRecoverySubject,
}

/// Reconciles a restored Vault A band snapshot against independently protected recovery
/// evidence (`25-availability-and-disaster-recovery.md`):
/// - Takes the maximum of `snapshot_high_water_counter` and `observed_high_water_counter`
///   so captured NFC tap proofs cannot be replayed after a database restore.
/// - Preserves terminal (`Retired`, `Failed`) or restrictive (`Frozen`, `ReplacementPending`)
///   band statuses even if the older database backup showed `Active`.
/// - Advances `post_restore_revocation_version` strictly above both the snapshot and evidence
///   versions so all pre-restore `VerifiedClinicSession` capabilities are invalidated.
/// - Quarantines the band into `BandStatus::Frozen` if `evidence_verified` is false.
pub fn reconcile_band_restore(
    snapshot: &VaultABandBackupSnapshot,
    evidence: &ProtectedRecoveryEvidence,
    tag_keys: Ntag424TagKeys,
) -> Result<ReconciledBandRecovery, RecoveryError> {
    if snapshot.blind_patient_id != evidence.blind_patient_id
        || snapshot.tag_uid != evidence.tag_uid
        || tag_keys.expected_uid != snapshot.tag_uid
    {
        return Err(RecoveryError::MismatchedRecoverySubject);
    }

    let reconciled_high_water = snapshot
        .snapshot_high_water_counter
        .max(evidence.observed_high_water_counter);

    let (reconciled_status, quarantined) = if !evidence.evidence_verified {
        // Fail closed when counter/status evidence cannot be established after restore.
        let safe_status = match (
            snapshot.snapshot_band_status,
            evidence.latest_known_band_status,
        ) {
            (BandStatus::Retired, _) | (_, BandStatus::Retired) => BandStatus::Retired,
            (BandStatus::Failed, _) | (_, BandStatus::Failed) => BandStatus::Failed,
            _ => BandStatus::Frozen,
        };
        (safe_status, true)
    } else {
        (
            most_restrictive_band_status(
                snapshot.snapshot_band_status,
                evidence.latest_known_band_status,
            ),
            false,
        )
    };

    // Always bump revocation_version past any pre-restore session's snapshot version.
    let post_restore_revocation_version = snapshot
        .snapshot_revocation_version
        .max(evidence.observed_revocation_version)
        .saturating_add(1);

    let restored_band_state = DurableBandReplayState::new(
        snapshot.blind_patient_id.clone(),
        tag_keys,
        reconciled_status,
        reconciled_high_water,
    );

    Ok(ReconciledBandRecovery {
        restored_band_state,
        post_restore_revocation_version,
        quarantined_for_manual_verification: quarantined,
    })
}

fn most_restrictive_band_status(a: BandStatus, b: BandStatus) -> BandStatus {
    if a == BandStatus::Retired || b == BandStatus::Retired {
        return BandStatus::Retired;
    }
    if a == BandStatus::Failed || b == BandStatus::Failed {
        return BandStatus::Failed;
    }
    if a == BandStatus::ReplacementPending || b == BandStatus::ReplacementPending {
        return BandStatus::ReplacementPending;
    }
    if a == BandStatus::Frozen || b == BandStatus::Frozen {
        return BandStatus::Frozen;
    }
    if a == BandStatus::VerificationPending || b == BandStatus::VerificationPending {
        return BandStatus::VerificationPending;
    }
    if a == BandStatus::Provisioning || b == BandStatus::Provisioning {
        return BandStatus::Provisioning;
    }
    if a == BandStatus::Inventory || b == BandStatus::Inventory {
        return BandStatus::Inventory;
    }
    BandStatus::Active
}
