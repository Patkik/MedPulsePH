import { useState } from 'react';
import { Pressable, StyleSheet, Text, useWindowDimensions, View } from 'react-native';
import { AppShell } from '../components/AppShell';

export default function AdminAndAuditScreen() {
  const [selfApproveBlocked, setSelfApproveBlocked] = useState<boolean>(false);
  const [doctorSuspended, setDoctorSuspended] = useState<boolean>(false);
  const [exportAttemptBlocked, setExportAttemptBlocked] = useState<boolean>(false);
  const { width } = useWindowDimensions();
  const compact = width > 0 && width < 860;

  return (
    <AppShell
      screenCode="S09 & S10 · Administration, Credentials & Privacy Audit"
      title="Hospital staff administration, independent credential review, and minimized privacy audit"
      subtitle="Hospital Administrators manage memberships, Credential Reviewers approve clinical eligibility independently (never self-approving), and Privacy Auditors inspect minimized immutable_audit_logs metadata without access to clinical notes or Vault A PII."
      roleBadge="Hospital Admin (S09) · Credential Reviewer (S09) · Privacy Auditor (S10)"
    >
      <View style={[styles.columns, compact && styles.columnsCompact]}>
        <View style={styles.card}>
          <Text style={styles.cardTitle}>S09 · Staff roster & independent credential review</Text>
          <Text style={styles.cardSub}>
            Self-approval of clinical eligibility is rejected by AuthError::SelfApprovalForbidden. Suspending a clinician advances revocation_version to stop in-flight clinical mutations.
          </Text>

          <View style={styles.staffItem}>
            <View style={styles.staffTop}>
              <Text style={styles.staffName}>Dr. Alonzo Reyes (doc-101 · Role::Doctor)</Text>
              <Text style={[styles.badge, doctorSuspended && styles.badgeCritical]}>
                {doctorSuspended ? 'Membership: SUSPENDED' : 'Approved by cred-rev-1'}
              </Text>
            </View>
            <Text style={styles.staffMeta}>
              Hospital: hosp-manila · PRC License verified · MFA enrolled
            </Text>
            <View style={styles.btnRow}>
              <Pressable
                accessibilityRole="button"
                onPress={() => setDoctorSuspended((s) => !s)}
                style={[styles.actionBtn, !doctorSuspended && styles.actionBtnDanger]}
              >
                <Text style={styles.actionBtnText}>
                  {doctorSuspended ? 'Restore active membership' : 'Suspend membership (Revokes active sessions)'}
                </Text>
              </Pressable>
              <Pressable
                accessibilityRole="button"
                onPress={() => setSelfApproveBlocked(true)}
                style={styles.secondaryBtn}
              >
                <Text style={styles.secondaryBtnText}>
                  Simulate self-approval (actor == subject)
                </Text>
              </Pressable>
            </View>
          </View>

          {selfApproveBlocked && (
            <View style={styles.errorBox} accessibilityRole="alert">
              <Text style={styles.errorTitle}>
                Denied · AuthError::SelfApprovalForbidden
              </Text>
              <Text style={styles.errorBody}>
                backend/src/auth.rs blocks any credential approval where reviewer_actor_id == subject_actor_id, as well as any doctor presenting a self-approved credential.
              </Text>
            </View>
          )}

          <View style={styles.staffItem}>
            <View style={styles.staffTop}>
              <Text style={styles.staffName}>Nurse Carla Santos (care-1 · Role::Nurse)</Text>
              <Text style={styles.badgeMuted}>Pilot disabled (D-06)</Text>
            </View>
            <Text style={styles.staffMeta}>
              CareTeamClinicalAccessDisabled enforced until hospital care-team authorization model is approved.
            </Text>
          </View>
        </View>

        <View style={styles.card}>
          <Text style={styles.cardTitle}>S10 · Minimized privacy auditor projection</Text>
          <Text style={styles.cardSub}>
            project_minimized_audit_for_privacy_auditor (backend/src/encounter.rs) returns only pseudonymous metadata for hosp-manila. Clinical note ciphertexts, plaintexts, and Vault A names are omitted.
          </Text>

          <View style={styles.auditTable}>
            {[
              {
                action: 'encounter.amend',
                patient: 'bp_v1_9f4a8d2c...0706',
                actor: 'doc-101',
                session: 'sess-manila-1',
                time: '1700000100',
              },
              {
                action: 'encounter.finalize',
                patient: 'bp_v1_9f4a8d2c...0706',
                actor: 'doc-101',
                session: 'sess-manila-1',
                time: '1700000080',
              },
              {
                action: 'encounter.review',
                patient: 'bp_v1_9f4a8d2c...0706',
                actor: 'doc-101',
                session: 'sess-manila-1',
                time: '1700000040',
              },
              {
                action: 'encounter.create_draft',
                patient: 'bp_v1_9f4a8d2c...0706',
                actor: 'doc-101',
                session: 'sess-manila-1',
                time: '1700000010',
              },
            ].map((row, idx) => (
              <View key={idx} style={styles.auditRow}>
                <View style={styles.staffTop}>
                  <Text style={styles.auditAction}>{row.action}</Text>
                  <Text style={styles.auditTime}>unix: {row.time}</Text>
                </View>
                <Text style={styles.auditMeta}>
                  Blind ID: {row.patient} · Actor: {row.actor} · Session: {row.session}
                </Text>
              </View>
            ))}
          </View>

          <View style={styles.btnRow}>
            <Pressable
              accessibilityRole="button"
              onPress={() => setExportAttemptBlocked(true)}
              style={styles.secondaryBtn}
            >
              <Text style={styles.secondaryBtnText}>
                Attempt bulk clinical record export ($export)
              </Text>
            </Pressable>
          </View>

          {exportAttemptBlocked && (
            <View style={styles.errorBox} accessibilityRole="alert">
              <Text style={styles.errorTitle}>
                Denied · ProtectedAction::ExportClinicalRecords / FhirError::DeferredOperationDisabled
              </Text>
              <Text style={styles.errorBody}>
                Bulk clinical exports ($export) and unconstrained graph queries (_include=*) are explicitly disabled in the pilot baseline.
              </Text>
            </View>
          )}
        </View>
      </View>
    </AppShell>
  );
}

const styles = StyleSheet.create({
  columns: { flexDirection: 'row', alignItems: 'flex-start', gap: 18 },
  columnsCompact: { flexDirection: 'column' },
  card: {
    flex: 1,
    width: '100%',
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    borderWidth: 1,
    borderColor: '#C7D5D0',
    padding: 20,
    gap: 14,
  },
  cardTitle: { color: '#15362F', fontSize: 18, fontWeight: '700' },
  cardSub: { color: '#4A675D', fontSize: 13, lineHeight: 20 },
  staffItem: {
    backgroundColor: '#F5F8F7',
    borderRadius: 10,
    borderWidth: 1,
    borderColor: '#C7D5D0',
    padding: 14,
    gap: 8,
  },
  staffTop: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    justifyContent: 'space-between',
    alignItems: 'center',
    gap: 8,
  },
  staffName: { color: '#15362F', fontSize: 14, fontWeight: '700' },
  staffMeta: { color: '#4A675D', fontSize: 12 },
  badge: {
    backgroundColor: '#E7EFEA',
    color: '#176856',
    fontSize: 11,
    fontWeight: '700',
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 6,
  },
  badgeCritical: { backgroundColor: '#FCECEE', color: '#B42332' },
  badgeMuted: {
    backgroundColor: '#E7EFEA',
    color: '#4A675D',
    fontSize: 11,
    fontWeight: '700',
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 6,
  },
  btnRow: { flexDirection: 'row', flexWrap: 'wrap', gap: 8 },
  actionBtn: {
    backgroundColor: '#176856',
    paddingHorizontal: 13,
    paddingVertical: 9,
    borderRadius: 8,
  },
  actionBtnDanger: { backgroundColor: '#B42332' },
  actionBtnText: { color: '#FFFFFF', fontSize: 12, fontWeight: '700' },
  secondaryBtn: {
    backgroundColor: '#FFFFFF',
    borderWidth: 1,
    borderColor: '#C7D5D0',
    paddingHorizontal: 13,
    paddingVertical: 9,
    borderRadius: 8,
  },
  secondaryBtnText: { color: '#15362F', fontSize: 12, fontWeight: '600' },
  errorBox: {
    backgroundColor: '#FCECEE',
    borderWidth: 1,
    borderColor: '#D9828B',
    borderRadius: 8,
    padding: 12,
    gap: 4,
  },
  errorTitle: { color: '#B42332', fontSize: 13, fontWeight: '700' },
  errorBody: { color: '#15362F', fontSize: 12, lineHeight: 18 },
  auditTable: { gap: 8 },
  auditRow: {
    backgroundColor: '#F5F8F7',
    borderRadius: 8,
    borderWidth: 1,
    borderColor: '#C7D5D0',
    padding: 11,
    gap: 3,
  },
  auditAction: { color: '#15362F', fontSize: 13, fontWeight: '700' },
  auditTime: { color: '#4A675D', fontSize: 11, fontWeight: '600' },
  auditMeta: { color: '#4A675D', fontSize: 12 },
});
