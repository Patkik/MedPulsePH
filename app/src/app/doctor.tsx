import { useState } from 'react';
import { Pressable, StyleSheet, Text, useWindowDimensions, View } from 'react-native';
import { AppShell } from '../components/AppShell';

type EncounterStage = 'draft' | 'reviewed' | 'finalized' | 'amended';

interface RevisionEntry {
  version: number;
  stage: EncounterStage;
  note: string;
  reason?: string;
}

export default function DoctorEncounterScreen() {
  const [pairedWorkstation, setPairedWorkstation] = useState<boolean>(true);
  const [sessionActive, setSessionActive] = useState<boolean>(true);
  const [failedAttempts, setFailedAttempts] = useState<number>(0);
  const [bandFrozenDuringSession, setBandFrozenDuringSession] = useState<boolean>(false);
  const [connectionLost, setConnectionLost] = useState<boolean>(false);
  const [auditOutage, setAuditOutage] = useState<boolean>(false);
  const [revisions, setRevisions] = useState<RevisionEntry[]>([
    {
      version: 1,
      stage: 'draft',
      note: 'Synthetic outpatient follow-up: BP 118/76 mmHg, HR 72 bpm. No acute cardiopulmonary distress. Verified Penicillin & Cefazolin beta-lactam allergy.',
    },
  ]);

  const { width } = useWindowDimensions();
  const compact = width > 0 && width < 860;

  const latestRevision = revisions[revisions.length - 1];
  const sessionLockedByBoundary = !sessionActive || bandFrozenDuringSession || auditOutage;

  function verifySyntheticPin(valid: boolean) {
    if (failedAttempts >= 5) return;
    if (valid) {
      setSessionActive(true);
      setFailedAttempts(0);
    } else {
      const next = failedAttempts + 1;
      setFailedAttempts(next);
      if (next >= 5) {
        setSessionActive(false);
      }
    }
  }

  function advanceEncounter(targetStage: EncounterStage) {
    if (sessionLockedByBoundary || connectionLost) return;
    const nextVersion = latestRevision.version + 1;
    if (targetStage === 'reviewed') {
      setRevisions((prev) => [
        ...prev,
        {
          version: nextVersion,
          stage: 'reviewed',
          note: `${latestRevision.note} [Reviewed by Dr. Alonzo Reyes · doc-101]`,
        },
      ]);
    } else if (targetStage === 'finalized') {
      setRevisions((prev) => [
        ...prev,
        {
          version: nextVersion,
          stage: 'finalized',
          note: 'Finalized synthetic encounter summary: Continue non-beta-lactam regimen; return in 14 days.',
        },
      ]);
    } else if (targetStage === 'amended') {
      setRevisions((prev) => [
        ...prev,
        {
          version: nextVersion,
          stage: 'amended',
          note: 'Finalized synthetic encounter summary + Addendum: Synthetic serum potassium recheck normal (4.2 mmol/L).',
          reason: 'Addendum: post-visit laboratory potassium value attached with provenance',
        },
      ]);
    }
  }

  return (
    <AppShell
      screenCode="S06, S07 & S08 · Doctor Clinical Workspace"
      title="Workstation tap pairing, dual-custody PIN handshake, and 15-minute clinical encounter"
      subtitle="Full clinical access (Tier 2) requires both an accredited doctor's session and the patient's 6-digit PIN/OTP to mint a fixed 15-minute VerifiedClinicSession. Every read or write revalidates revocation state and commits an immutable audit log entry."
      roleBadge="Dr. Alonzo Reyes (doc-101 · Accredited Physician · hosp-manila)"
    >
      <View style={[styles.topGrid, compact && styles.topGridCompact]}>
        <View style={styles.stageCard}>
          <Text style={styles.stageTitle}>S06 · Workstation pairing & tap context</Text>
          <Text style={styles.stageSub}>
            Single-use pairing token binds phone NFC reader tap counter #14 to workstation ws-opd-04. Pairing QR alone never opens a chart without cryptographic SUN proof.
          </Text>
          <View style={styles.metaBox}>
            <Text style={styles.metaLine}>Workstation: ws-opd-04 · Hospital: hosp-manila</Text>
            <Text style={styles.metaLine}>Bound tap context: tap-ctx-014 (Counter #14 accepted)</Text>
            <Text style={styles.metaLine}>
              Status: {pairedWorkstation ? 'Paired & bound to doc-101' : 'Pairing expired'}
            </Text>
          </View>
          <Pressable
            accessibilityRole="button"
            onPress={() => setPairedWorkstation((p) => !p)}
            style={styles.secondaryBtn}
          >
            <Text style={styles.secondaryBtnText}>
              {pairedWorkstation ? 'Simulate expired pairing token' : 'Re-issue workstation pairing (120s TTL)'}
            </Text>
          </Pressable>
        </View>

        <View style={styles.stageCard}>
          <Text style={styles.stageTitle}>S07 · Dual-custody 6-digit patient PIN handshake</Text>
          <Text style={styles.stageSub}>
            Argon2id PIN verification consumes AcceptedTapContext once and mints VerifiedClinicSession (900s fixed duration). 5 failed attempts revoke the tap context.
          </Text>
          <View style={styles.metaBox}>
            <Text style={styles.metaLine}>
              Session state: {sessionActive ? 'VerifiedClinicSession active (14m 42s remaining)' : 'Locked — awaiting patient PIN'}
            </Text>
            <Text style={styles.metaLine}>
              Failed PIN attempts: {failedAttempts} / 5 {failedAttempts >= 5 ? '(Revoked — tap again)' : ''}
            </Text>
          </View>
          <View style={styles.btnRow}>
            <Pressable
              accessibilityRole="button"
              onPress={() => verifySyntheticPin(true)}
              style={[styles.primaryBtn, failedAttempts >= 5 && styles.btnDisabled]}
            >
              <Text style={styles.primaryBtnText}>Verify valid patient PIN (482910)</Text>
            </Pressable>
            <Pressable
              accessibilityRole="button"
              onPress={() => verifySyntheticPin(false)}
              style={styles.secondaryBtn}
            >
              <Text style={styles.secondaryBtnText}>Simulate wrong PIN</Text>
            </Pressable>
            {failedAttempts > 0 && (
              <Pressable
                accessibilityRole="button"
                onPress={() => {
                  setFailedAttempts(0);
                  setSessionActive(true);
                }}
                style={styles.secondaryBtn}
              >
                <Text style={styles.secondaryBtnText}>Fresh NFC tap (#15)</Text>
              </Pressable>
            )}
          </View>
        </View>
      </View>

      <View style={styles.workspaceCard}>
        <View style={styles.patientStrip}>
          <View style={styles.patientStripLeft}>
            <Text style={styles.patientName}>
              Maria Synthetic Dela Cruz · FHIR Patient/{'bp_v1_9f4a8d2c1b0e7a6f5d4c3b2a19080706'}
            </Text>
            <Text style={styles.patientSub}>
              Assembled at API boundary from Vault A · Vault B stores only BlindPatientId & AES-256-GCM ciphertexts
            </Text>
          </View>
          <View style={[styles.timerBadge, sessionLockedByBoundary && styles.timerBadgeLocked]}>
            <Text style={[styles.timerText, sessionLockedByBoundary && styles.timerTextLocked]}>
              {bandFrozenDuringSession
                ? 'LOCKED · BandFrozen'
                : auditOutage
                  ? 'LOCKED · AuditStorageUnavailable'
                  : sessionActive
                    ? '14:42 / 15:00 remaining'
                    : 'SESSION LOCKED'}
            </Text>
          </View>
        </View>

        <View style={styles.faultBar}>
          <Text style={styles.faultLabel}>Transaction-boundary resilience checks:</Text>
          <Pressable
            accessibilityRole="button"
            onPress={() => setBandFrozenDuringSession((b) => !b)}
            style={[styles.faultChip, bandFrozenDuringSession && styles.faultChipActive]}
          >
            <Text style={[styles.faultChipText, bandFrozenDuringSession && styles.faultChipTextActive]}>
              {bandFrozenDuringSession ? 'Band frozen (Active)' : 'Simulate patient band freeze'}
            </Text>
          </Pressable>
          <Pressable
            accessibilityRole="button"
            onPress={() => setConnectionLost((c) => !c)}
            style={[styles.faultChip, connectionLost && styles.faultChipWarn]}
          >
            <Text style={[styles.faultChipText, connectionLost && styles.faultChipTextActive]}>
              {connectionLost ? 'Offline (Saves blocked)' : 'Simulate connection loss'}
            </Text>
          </Pressable>
          <Pressable
            accessibilityRole="button"
            onPress={() => setAuditOutage((a) => !a)}
            style={[styles.faultChip, auditOutage && styles.faultChipActive]}
          >
            <Text style={[styles.faultChipText, auditOutage && styles.faultChipTextActive]}>
              {auditOutage ? 'Audit store outage (Fail-closed)' : 'Simulate audit-store outage'}
            </Text>
          </Pressable>
        </View>

        {sessionLockedByBoundary ? (
          <View style={styles.lockedBanner} accessibilityRole="alert">
            <Text style={styles.lockedTitle}>
              Clinical workspace locked at transaction boundary
            </Text>
            <Text style={styles.lockedBody}>
              {bandFrozenDuringSession
                ? 'The patient froze their wristband (BandStatus::Frozen). revalidate_at_transaction_boundary rejected further reads and writes immediately.'
                : auditOutage
                  ? 'Vault B durable audit storage is unreachable (EncounterError::AuditStorageUnavailable). Clinical reads and writes fail closed rather than bypassing immutable audit.'
                  : 'VerifiedClinicSession is not active. Complete the NTAG 424 DNA tap and 6-digit patient PIN handshake to open the chart.'}
            </Text>
          </View>
        ) : (
          <View style={styles.encounterBody}>
            {connectionLost && (
              <View style={styles.offlineBanner} accessibilityRole="alert">
                <Text style={styles.offlineTitle}>
                  Connection lost · Clinical writes paused
                </Text>
                <Text style={styles.offlineBody}>
                  The interface never claims an unconfirmed write succeeded and never caches plaintext clinical drafts in browser localStorage.
                </Text>
              </View>
            )}

            <View style={styles.encounterHeader}>
              <Text style={styles.encounterTitle}>
                S08 · Encounter enc-1001 · State: {latestRevision.stage.toUpperCase()} · ResourceVersion v{latestRevision.version}
              </Text>
              <Text style={styles.encounterMeta}>
                AES-256-GCM Key: vault-b-enc-k1 · AAD bound to (BlindPatientId, EncounterNote, v{latestRevision.version})
              </Text>
            </View>

            <View style={styles.noteBox}>
              <Text style={styles.noteText}>{latestRevision.note}</Text>
              {latestRevision.reason && (
                <Text style={styles.amendReason}>
                  Amendment provenance: {latestRevision.reason}
                </Text>
              )}
            </View>

            <View style={styles.btnRow}>
              {latestRevision.stage === 'draft' && (
                <Pressable
                  accessibilityRole="button"
                  onPress={() => advanceEncounter('reviewed')}
                  style={[styles.primaryBtn, connectionLost && styles.btnDisabled]}
                >
                  <Text style={styles.primaryBtnText}>Mark draft reviewed (v{latestRevision.version + 1})</Text>
                </Pressable>
              )}
              {(latestRevision.stage === 'draft' || latestRevision.stage === 'reviewed') && (
                <Pressable
                  accessibilityRole="button"
                  onPress={() => advanceEncounter('finalized')}
                  style={[styles.primaryBtn, connectionLost && styles.btnDisabled]}
                >
                  <Text style={styles.primaryBtnText}>Finalize encounter (Locks in-place edits)</Text>
                </Pressable>
              )}
              {(latestRevision.stage === 'finalized' || latestRevision.stage === 'amended') && (
                <Pressable
                  accessibilityRole="button"
                  onPress={() => advanceEncounter('amended')}
                  style={[styles.primaryBtn, connectionLost && styles.btnDisabled]}
                >
                  <Text style={styles.primaryBtnText}>Append traceable amendment (v{latestRevision.version + 1})</Text>
                </Pressable>
              )}
              {revisions.length > 1 && (
                <Pressable
                  accessibilityRole="button"
                  onPress={() => setRevisions([revisions[0]])}
                  style={styles.secondaryBtn}
                >
                  <Text style={styles.secondaryBtnText}>Reset encounter demo to v1 Draft</Text>
                </Pressable>
              )}
            </View>

            <View style={styles.revisionHistory}>
              <Text style={styles.revisionHeading}>
                Immutable AES-256-GCM revision & audit trail ({revisions.length} revisions preserved):
              </Text>
              {revisions.map((rev) => (
                <Text key={rev.version} style={styles.revisionItem}>
                  • v{rev.version} [{rev.stage}] — {rev.note}
                </Text>
              ))}
            </View>
          </View>
        )}
      </View>
    </AppShell>
  );
}

const styles = StyleSheet.create({
  topGrid: { flexDirection: 'row', alignItems: 'stretch', gap: 16 },
  topGridCompact: { flexDirection: 'column' },
  stageCard: {
    flex: 1,
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    borderWidth: 1,
    borderColor: '#C7D5D0',
    padding: 18,
    gap: 10,
  },
  stageTitle: { color: '#15362F', fontSize: 16, fontWeight: '700' },
  stageSub: { color: '#4A675D', fontSize: 13, lineHeight: 19 },
  metaBox: {
    backgroundColor: '#F5F8F7',
    borderRadius: 8,
    borderWidth: 1,
    borderColor: '#C7D5D0',
    padding: 12,
    gap: 4,
  },
  metaLine: { color: '#15362F', fontSize: 12, fontWeight: '600' },
  btnRow: { flexDirection: 'row', flexWrap: 'wrap', gap: 8 },
  primaryBtn: {
    backgroundColor: '#176856',
    paddingHorizontal: 14,
    paddingVertical: 10,
    borderRadius: 8,
    minHeight: 40,
    justifyContent: 'center',
  },
  btnDisabled: { backgroundColor: '#8AA39B' },
  primaryBtnText: { color: '#FFFFFF', fontSize: 13, fontWeight: '700' },
  secondaryBtn: {
    backgroundColor: '#FFFFFF',
    borderWidth: 1,
    borderColor: '#C7D5D0',
    paddingHorizontal: 14,
    paddingVertical: 10,
    borderRadius: 8,
    minHeight: 40,
    justifyContent: 'center',
    alignSelf: 'flex-start',
  },
  secondaryBtnText: { color: '#15362F', fontSize: 13, fontWeight: '600' },
  workspaceCard: {
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    borderWidth: 1,
    borderColor: '#C7D5D0',
    padding: 20,
    gap: 16,
  },
  patientStrip: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    justifyContent: 'space-between',
    alignItems: 'center',
    gap: 12,
    paddingBottom: 14,
    borderBottomWidth: 1,
    borderBottomColor: '#C7D5D0',
  },
  patientStripLeft: { flex: 1, minWidth: 240, gap: 4 },
  patientName: { color: '#15362F', fontSize: 16, fontWeight: '700' },
  patientSub: { color: '#4A675D', fontSize: 12 },
  timerBadge: {
    backgroundColor: '#E7EFEA',
    borderWidth: 1,
    borderColor: '#8AB5A5',
    paddingHorizontal: 12,
    paddingVertical: 7,
    borderRadius: 8,
  },
  timerBadgeLocked: { backgroundColor: '#FCECEE', borderColor: '#D9828B' },
  timerText: { color: '#15362F', fontSize: 13, fontWeight: '700' },
  timerTextLocked: { color: '#B42332' },
  faultBar: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    alignItems: 'center',
    gap: 8,
  },
  faultLabel: { color: '#4A675D', fontSize: 12, fontWeight: '700' },
  faultChip: {
    paddingHorizontal: 11,
    paddingVertical: 7,
    borderRadius: 6,
    borderWidth: 1,
    borderColor: '#C7D5D0',
    backgroundColor: '#F5F8F7',
  },
  faultChipActive: { backgroundColor: '#B42332', borderColor: '#B42332' },
  faultChipWarn: { backgroundColor: '#8B5C00', borderColor: '#8B5C00' },
  faultChipText: { color: '#15362F', fontSize: 12, fontWeight: '600' },
  faultChipTextActive: { color: '#FFFFFF' },
  lockedBanner: {
    backgroundColor: '#FCECEE',
    borderWidth: 1,
    borderColor: '#D9828B',
    borderRadius: 10,
    padding: 16,
    gap: 6,
  },
  lockedTitle: { color: '#B42332', fontSize: 16, fontWeight: '700' },
  lockedBody: { color: '#15362F', fontSize: 13, lineHeight: 20 },
  offlineBanner: {
    backgroundColor: '#FDF6E7',
    borderWidth: 1,
    borderColor: '#D6AE5C',
    borderRadius: 10,
    padding: 14,
    gap: 4,
  },
  offlineTitle: { color: '#8B5C00', fontSize: 14, fontWeight: '700' },
  offlineBody: { color: '#15362F', fontSize: 13, lineHeight: 19 },
  encounterBody: { gap: 14 },
  encounterHeader: { gap: 4 },
  encounterTitle: { color: '#15362F', fontSize: 16, fontWeight: '700' },
  encounterMeta: { color: '#4A675D', fontSize: 12 },
  noteBox: {
    backgroundColor: '#F5F8F7',
    borderRadius: 10,
    borderWidth: 1,
    borderColor: '#C7D5D0',
    padding: 16,
    gap: 8,
  },
  noteText: { color: '#15362F', fontSize: 14, lineHeight: 22 },
  amendReason: { color: '#176856', fontSize: 12, fontWeight: '700' },
  revisionHistory: {
    backgroundColor: '#E7EFEA',
    borderRadius: 10,
    padding: 14,
    gap: 6,
  },
  revisionHeading: { color: '#15362F', fontSize: 13, fontWeight: '700' },
  revisionItem: { color: '#15362F', fontSize: 12, lineHeight: 18 },
});
