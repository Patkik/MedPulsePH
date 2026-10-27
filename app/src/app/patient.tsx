import { useState } from 'react';
import { Pressable, StyleSheet, Text, useWindowDimensions, View } from 'react-native';
import { AppShell } from '../components/AppShell';

type ConsentDecision = 'pending' | 'approved' | 'declined';

export default function PatientDashboardScreen() {
  const [bandFrozen, setBandFrozen] = useState<boolean>(false);
  const [consentDecision, setConsentDecision] = useState<ConsentDecision>('pending');
  const [sharingHospCebu, setSharingHospCebu] = useState<boolean>(false);
  const { width } = useWindowDimensions();
  const compact = width > 0 && width < 840;

  return (
    <AppShell
      screenCode="S05 · Patient Band, Consent & Activity"
      title="Your wristband, clinical consent requests, and access log"
      subtitle="Control whether your wristband is active or frozen, review clinician requests before entering your 6-digit PIN, and inspect every recorded access to your record."
      roleBadge="Patient · Self-service or hospital station"
    >
      <View style={[styles.columns, compact && styles.columnsCompact]}>
        <View style={styles.card}>
          <View style={styles.cardHeader}>
            <View>
              <Text style={styles.cardTitle}>Physical NFC wristband status</Text>
              <Text style={styles.cardSub}>UID 04:91:82:73:64:55:46 · NTAG 424 DNA</Text>
            </View>
            <View style={[styles.statusPill, bandFrozen ? styles.statusPillFrozen : styles.statusPillActive]}>
              <Text style={[styles.statusPillText, bandFrozen && styles.statusPillTextFrozen]}>
                {bandFrozen ? 'FROZEN (403 Locked)' : 'ACTIVE'}
              </Text>
            </View>
          </View>

          <Text style={styles.bodyText}>
            {bandFrozen
              ? 'Your wristband is currently frozen. Both Tier-1 emergency taps and Tier-2 clinical sessions are blocked immediately at the transaction boundary. Freezing your band never deletes your medical records.'
              : 'Your wristband is active. Tapping it in an emergency shows only your blood type, critical allergies, and ICE contact. Full clinical chart access requires an accredited doctor and your 6-digit PIN.'}
          </Text>

          <Pressable
            accessibilityRole="button"
            onPress={() => setBandFrozen((f) => !f)}
            style={[styles.freezeBtn, bandFrozen && styles.unfreezeBtn]}
          >
            <Text style={styles.freezeBtnText}>
              {bandFrozen ? 'Unfreeze wristband (Restore active status)' : 'Freeze wristband immediately'}
            </Text>
          </Pressable>

          <View style={styles.requestCard}>
            <Text style={styles.subHeading}>Pending clinical encounter request</Text>
            <Text style={styles.requestMeta}>
              Dr. Alonzo Reyes (Accredited Physician · doc-101) at Philippine General Synthetic Medical Center (hosp-manila)
            </Text>
            <Text style={styles.requestScope}>
              Requested scope: Read & update outpatient clinical encounter · Fixed 15-minute session (900 seconds)
            </Text>

            {bandFrozen ? (
              <Text style={styles.declinedNotice}>
                Blocked because your wristband is frozen (AuthError::BandNotActive).
              </Text>
            ) : consentDecision === 'pending' ? (
              <View style={styles.btnRow}>
                <Pressable
                  accessibilityRole="button"
                  onPress={() => setConsentDecision('approved')}
                  style={styles.approveBtn}
                >
                  <Text style={styles.approveBtnText}>Approve with 6-digit PIN (15 min)</Text>
                </Pressable>
                <Pressable
                  accessibilityRole="button"
                  onPress={() => setConsentDecision('declined')}
                  style={styles.declineBtn}
                >
                  <Text style={styles.declineBtnText}>Decline request</Text>
                </Pressable>
              </View>
            ) : consentDecision === 'approved' ? (
              <View style={styles.outcomeBox}>
                <Text style={styles.outcomeOk}>
                  Approved · VerifiedClinicSession minted for 15 minutes (sess-001).
                </Text>
                <Pressable onPress={() => setConsentDecision('pending')} style={styles.resetLink}>
                  <Text style={styles.resetLinkText}>Reset request state</Text>
                </Pressable>
              </View>
            ) : (
              <View style={styles.outcomeBox}>
                <Text style={styles.declinedNotice}>
                  Request declined (AuthorizationRequestState::Declined). The session was not opened and the doctor will not be prompted repeatedly.
                </Text>
                <Pressable onPress={() => setConsentDecision('pending')} style={styles.resetLink}>
                  <Text style={styles.resetLinkText}>Reset request state</Text>
                </Pressable>
              </View>
            )}
          </View>
        </View>

        <View style={styles.card}>
          <Text style={styles.cardTitle}>Cross-hospital sharing & access history</Text>
          <Text style={styles.cardSub}>
            Records authored at one hospital cannot be read by another hospital unless you authorize longitudinal sharing (HospitalSharingConsent).
          </Text>

          <View style={styles.sharingRow}>
            <View style={styles.sharingTextWrap}>
              <Text style={styles.subHeading}>Share longitudinal records with Vicente Sotto (hosp-cebu)</Text>
              <Text style={styles.cardSub}>
                {sharingHospCebu
                  ? 'ExplicitHospitalList(["hosp-cebu"]) active — accredited Cebu doctors with your PIN can view PGH Manila encounters.'
                  : 'SourceHospitalOnly — Cebu sessions receive EncounterError::CrossHospitalRecordNotShared.'}
              </Text>
            </View>
            <Pressable
              accessibilityRole="button"
              onPress={() => setSharingHospCebu((s) => !s)}
              style={[styles.toggleBtn, sharingHospCebu && styles.toggleBtnOn]}
            >
              <Text style={styles.toggleBtnText}>
                {sharingHospCebu ? 'Shared' : 'Source only'}
              </Text>
            </Pressable>
          </View>

          <Text style={styles.subHeading}>Recorded disclosures (immutable_audit_logs)</Text>
          <View style={styles.logList}>
            {[
              {
                time: '2026-10-28 03:30 UTC',
                actor: 'Dr. Alonzo Reyes (doc-101 · hosp-manila)',
                action: 'encounter.finalize (enc-1001 · v3)',
                session: 'sess-manila-1 (PIN verified)',
              },
              {
                time: '2026-10-28 03:18 UTC',
                actor: 'Dr. Alonzo Reyes (doc-101 · hosp-manila)',
                action: 'encounter.read (enc-1001 · v1)',
                session: 'sess-manila-1 (PIN verified)',
              },
              {
                time: '2026-10-28 02:45 UTC',
                actor: 'Anonymous emergency tap (Tier 1)',
                action: 'emergency.triage_viewed (Blood type, allergies, ICE only)',
                session: 'tap-ctx-012 (Counter #12)',
              },
            ].map((item) => (
              <View key={item.time + item.action} style={styles.logItem}>
                <View style={styles.logTop}>
                  <Text style={styles.logAction}>{item.action}</Text>
                  <Text style={styles.logTime}>{item.time}</Text>
                </View>
                <Text style={styles.logMeta}>{item.actor}</Text>
                <Text style={styles.logMeta}>Context: {item.session}</Text>
              </View>
            ))}
          </View>
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
  cardHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
    gap: 10,
  },
  cardTitle: { color: '#15362F', fontSize: 18, fontWeight: '700' },
  cardSub: { color: '#4A675D', fontSize: 13, lineHeight: 19 },
  statusPill: {
    paddingHorizontal: 10,
    paddingVertical: 5,
    borderRadius: 6,
  },
  statusPillActive: { backgroundColor: '#E7EFEA' },
  statusPillFrozen: { backgroundColor: '#FCECEE' },
  statusPillText: { color: '#176856', fontSize: 12, fontWeight: '700' },
  statusPillTextFrozen: { color: '#B42332' },
  bodyText: { color: '#15362F', fontSize: 14, lineHeight: 21 },
  freezeBtn: {
    backgroundColor: '#B42332',
    paddingVertical: 12,
    paddingHorizontal: 16,
    borderRadius: 8,
    alignItems: 'center',
  },
  unfreezeBtn: { backgroundColor: '#176856' },
  freezeBtnText: { color: '#FFFFFF', fontSize: 13, fontWeight: '700' },
  requestCard: {
    backgroundColor: '#F5F8F7',
    borderRadius: 10,
    borderWidth: 1,
    borderColor: '#C7D5D0',
    padding: 16,
    gap: 8,
  },
  subHeading: { color: '#15362F', fontSize: 14, fontWeight: '700' },
  requestMeta: { color: '#15362F', fontSize: 13, fontWeight: '600' },
  requestScope: { color: '#4A675D', fontSize: 12, lineHeight: 18 },
  btnRow: { flexDirection: 'row', flexWrap: 'wrap', gap: 8, marginTop: 4 },
  approveBtn: {
    backgroundColor: '#176856',
    paddingHorizontal: 14,
    paddingVertical: 10,
    borderRadius: 8,
  },
  approveBtnText: { color: '#FFFFFF', fontSize: 13, fontWeight: '700' },
  declineBtn: {
    backgroundColor: '#FFFFFF',
    borderWidth: 1,
    borderColor: '#C7D5D0',
    paddingHorizontal: 14,
    paddingVertical: 10,
    borderRadius: 8,
  },
  declineBtnText: { color: '#15362F', fontSize: 13, fontWeight: '600' },
  outcomeBox: { gap: 6, marginTop: 4 },
  outcomeOk: { color: '#176856', fontSize: 13, fontWeight: '700' },
  declinedNotice: { color: '#B42332', fontSize: 13, fontWeight: '600' },
  resetLink: { alignSelf: 'flex-start' },
  resetLinkText: { color: '#15362F', fontSize: 12, textDecorationLine: 'underline' },
  sharingRow: {
    backgroundColor: '#F5F8F7',
    borderRadius: 10,
    borderWidth: 1,
    borderColor: '#C7D5D0',
    padding: 14,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    gap: 12,
  },
  sharingTextWrap: { flex: 1, minWidth: 0, gap: 4 },
  toggleBtn: {
    backgroundColor: '#4A675D',
    paddingHorizontal: 12,
    paddingVertical: 9,
    borderRadius: 8,
  },
  toggleBtnOn: { backgroundColor: '#176856' },
  toggleBtnText: { color: '#FFFFFF', fontSize: 12, fontWeight: '700' },
  logList: { gap: 10 },
  logItem: {
    backgroundColor: '#F5F8F7',
    borderRadius: 8,
    borderWidth: 1,
    borderColor: '#C7D5D0',
    padding: 12,
    gap: 3,
  },
  logTop: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    justifyContent: 'space-between',
    gap: 8,
  },
  logAction: { color: '#15362F', fontSize: 13, fontWeight: '700' },
  logTime: { color: '#4A675D', fontSize: 11, fontWeight: '600' },
  logMeta: { color: '#4A675D', fontSize: 12 },
});
