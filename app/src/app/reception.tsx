import { useState } from 'react';
import { Pressable, StyleSheet, Text, useWindowDimensions, View } from 'react-native';
import { AppShell } from '../components/AppShell';

type LinkState = 'proposed' | 'evidence_pending' | 'verified' | 'disputed';
type AllergyPreset = 'standard' | 'oversized';

export default function ReceptionAndBandsScreen() {
  const [linkState, setLinkState] = useState<LinkState>('verified');
  const [pinConfiguredByPatient, setPinConfiguredByPatient] = useState<boolean>(true);
  const [staffPinAttemptBlocked, setStaffPinAttemptBlocked] = useState<boolean>(false);
  const [allergyPreset, setAllergyPreset] = useState<AllergyPreset>('standard');
  const [bandGeneration, setBandGeneration] = useState<number>(1);
  const [retiredUids, setRetiredUids] = useState<string[]>([]);
  const { width } = useWindowDimensions();
  const compact = width > 0 && width < 860;

  const currentUid = bandGeneration === 1 ? '04:91:82:73:64:55:46' : `04:AA:BB:CC:DD:EE:0${bandGeneration}`;
  const record2Text =
    allergyPreset === 'standard'
      ? 'O+|Penicillin,Cefazolin|+639170000123|2026-10-28'
      : 'O+|Penicillin,Cefazolin,Sulfonamides,Carbamazepine,Phenytoin,Allopurinol,Lamotrigine,Abacavir,Nevirapine,Dapsone,Vancomycin|+639170000123|2026-10-28';
  const record2Bytes = record2Text.length;
  const record1Uri = 'https://medpulse.example/v1/tap?d=00000000000000000000000000000000&c=0000000000000000';
  const record1Bytes = 4 + 1 + (record1Uri.length - 'https://'.length);
  const totalNdefBytes = 2 + record1Bytes + (4 + 1 + 2 + record2Bytes);
  const budgetValid = record2Bytes <= 120 && totalNdefBytes <= 256;
  const canIssueOrReplace = linkState === 'verified' && pinConfiguredByPatient && budgetValid;

  function handleReplaceBand() {
    if (!canIssueOrReplace) return;
    setRetiredUids((prev) => [currentUid, ...prev]);
    setBandGeneration((g) => g + 1);
  }

  return (
    <AppShell
      screenCode="S03 & S04 · Reception & Band Operations"
      title="Assisted patient enrollment, identity verification, and NTAG 424 DNA band provisioning"
      subtitle="Reception prepares identity linkage and assisted registration without viewing clinical records or the patient's 6-digit PIN. Band Officers provision NTAG 424 DNA wristbands within the 256-byte NDEF budget and atomically retire replaced bands."
      roleBadge="Reception (S03) & Band Officer (S04) · Zero chart access"
    >
      <View style={[styles.columns, compact && styles.columnsCompact]}>
        <View style={styles.card}>
          <Text style={styles.cardTitle}>S03 · Patient identity link state machine</Text>
          <Text style={styles.cardSub}>
            Demographic similarity alone never links a record or binds a band. Only Verified links with recorded evidence allow PIN setup and band issuance (backend/src/enrollment.rs).
          </Text>

          <View style={styles.patientBanner}>
            <Text style={styles.patientName}>Maria Synthetic Dela Cruz (DOB 1988-04-12)</Text>
            <Text style={styles.patientMeta}>
              Blind ID: bp_v1_9f4a8d2c1b0e7a6f5d4c3b2a19080706 · Channel: AssistedNoPhoneAtHospital
            </Text>
          </View>

          <View style={styles.stateRow}>
            {(
              [
                ['proposed', 'Proposed'],
                ['evidence_pending', 'Evidence pending'],
                ['verified', 'Verified (ID checked)'],
                ['disputed', 'Disputed (Duplicate review)'],
              ] as [LinkState, string][]
            ).map(([key, label]) => {
              const active = linkState === key;
              return (
                <Pressable
                  key={key}
                  accessibilityRole="button"
                  accessibilityState={{ selected: active }}
                  onPress={() => setLinkState(key)}
                  style={[
                    styles.stateBtn,
                    active && (key === 'disputed' ? styles.stateBtnCritical : styles.stateBtnActive),
                  ]}
                >
                  <Text style={[styles.stateBtnText, active && styles.stateBtnTextActive]}>
                    {label}
                  </Text>
                </Pressable>
              );
            })}
          </View>

          <View style={styles.pinBoundaryCard}>
            <Text style={styles.subHeading}>Private patient-only 6-digit PIN boundary</Text>
            <Text style={styles.cardSub}>
              Reception can hand the patient a private keypad station, but staff roles are blocked by EnrollmentError::StaffCannotSetPatientPin.
            </Text>
            <View style={styles.actionRow}>
              <Pressable
                accessibilityRole="button"
                onPress={() => {
                  setStaffPinAttemptBlocked(false);
                  if (linkState === 'verified') {
                    setPinConfiguredByPatient(true);
                  }
                }}
                style={[styles.primaryBtn, linkState !== 'verified' && styles.btnDisabled]}
              >
                <Text style={styles.primaryBtnText}>
                  {pinConfiguredByPatient
                    ? 'Patient PIN configured on private keypad'
                    : 'Complete patient-held PIN entry'}
                </Text>
              </Pressable>

              <Pressable
                accessibilityRole="button"
                onPress={() => setStaffPinAttemptBlocked(true)}
                style={styles.secondaryBtn}
              >
                <Text style={styles.secondaryBtnText}>Simulate staff PIN entry attempt</Text>
              </Pressable>
            </View>

            {staffPinAttemptBlocked && (
              <View style={styles.errorAlert} accessibilityRole="alert">
                <Text style={styles.errorAlertTitle}>
                  Blocked · EnrollmentError::StaffCannotSetPatientPin
                </Text>
                <Text style={styles.errorAlertBody}>
                  Reception, Band Officer, Doctor, and Admin roles are forbidden from setting or reading a patient’s 6-digit clinical authorization PIN.
                </Text>
              </View>
            )}
          </View>
        </View>

        <View style={styles.card}>
          <Text style={styles.cardTitle}>S04 · NTAG 424 DNA 256-byte NDEF budget & replacement</Text>
          <Text style={styles.cardSub}>
            Record 1 holds the SUN dynamic URL; Record 2 holds compact offline triage text under 120 bytes. Oversized allergy lists halt provisioning rather than truncating allergies.
          </Text>

          <View style={styles.stateRow}>
            <Pressable
              accessibilityRole="button"
              onPress={() => setAllergyPreset('standard')}
              style={[styles.stateBtn, allergyPreset === 'standard' && styles.stateBtnActive]}
            >
              <Text style={[styles.stateBtnText, allergyPreset === 'standard' && styles.stateBtnTextActive]}>
                Standard allergies (48 B Record 2)
              </Text>
            </Pressable>
            <Pressable
              accessibilityRole="button"
              onPress={() => setAllergyPreset('oversized')}
              style={[styles.stateBtn, allergyPreset === 'oversized' && styles.stateBtnCritical]}
            >
              <Text style={[styles.stateBtnText, allergyPreset === 'oversized' && styles.stateBtnTextActive]}>
                Oversized allergy string ({'>'}120 B)
              </Text>
            </Pressable>
          </View>

          <View style={[styles.budgetBox, !budgetValid && styles.budgetBoxError]}>
            <View style={styles.budgetRow}>
              <Text style={styles.budgetLabel}>Record 2 offline text ({record2Bytes} / 120 B max)</Text>
              <Text style={styles.budgetLabel}>Total NDEF file ({totalNdefBytes} / 256 B max)</Text>
            </View>
            <Text style={styles.monoPayload}>{record2Text}</Text>
            <Text style={styles.budgetVerdict}>
              {budgetValid
                ? 'Pass · NdefDualRecordBudget::build_and_verify within 256-byte NTAG 424 DNA file capacity'
                : 'Blocked · NfcError::NdefBudgetExceeded — refuses to silently truncate critical allergies'}
            </Text>
          </View>

          <View style={styles.bandStatusBox}>
            <View style={styles.budgetRow}>
              <Text style={styles.subHeading}>Active wristband #{bandGeneration}</Text>
              <Text style={styles.uidBadge}>UID {currentUid}</Text>
            </View>
            <Text style={styles.cardSub}>
              Revocation version: v{bandGeneration + 1} · Durable high-water counter: 0
            </Text>

            <Pressable
              accessibilityRole="button"
              onPress={handleReplaceBand}
              style={[styles.primaryBtn, !canIssueOrReplace && styles.btnDisabled]}
            >
              <Text style={styles.primaryBtnText}>
                {canIssueOrReplace
                  ? 'Issue replacement band & atomically retire current band'
                  : 'Cannot replace band until identity is Verified and NDEF budget passes'}
              </Text>
            </Pressable>

            {retiredUids.length > 0 && (
              <View style={styles.retiredList}>
                <Text style={styles.retiredHeading}>
                  Permanently retired bands (online authority revoked immediately):
                </Text>
                {retiredUids.map((uid) => (
                  <Text key={uid} style={styles.retiredItem}>
                    • UID {uid} → BandStatus::Retired (cannot transition back to Active)
                  </Text>
                ))}
              </View>
            )}
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
  cardTitle: { color: '#15362F', fontSize: 18, fontWeight: '700' },
  cardSub: { color: '#4A675D', fontSize: 13, lineHeight: 20 },
  patientBanner: {
    backgroundColor: '#E7EFEA',
    borderRadius: 10,
    padding: 14,
    gap: 4,
  },
  patientName: { color: '#15362F', fontSize: 15, fontWeight: '700' },
  patientMeta: { color: '#4A675D', fontSize: 12 },
  stateRow: { flexDirection: 'row', flexWrap: 'wrap', gap: 8 },
  stateBtn: {
    paddingHorizontal: 12,
    paddingVertical: 9,
    borderRadius: 8,
    borderWidth: 1,
    borderColor: '#C7D5D0',
    backgroundColor: '#F5F8F7',
  },
  stateBtnActive: { backgroundColor: '#176856', borderColor: '#176856' },
  stateBtnCritical: { backgroundColor: '#B42332', borderColor: '#B42332' },
  stateBtnText: { color: '#15362F', fontSize: 12, fontWeight: '600' },
  stateBtnTextActive: { color: '#FFFFFF' },
  pinBoundaryCard: {
    backgroundColor: '#F5F8F7',
    borderRadius: 10,
    borderWidth: 1,
    borderColor: '#C7D5D0',
    padding: 14,
    gap: 10,
  },
  subHeading: { color: '#15362F', fontSize: 14, fontWeight: '700' },
  actionRow: { flexDirection: 'row', flexWrap: 'wrap', gap: 8 },
  primaryBtn: {
    backgroundColor: '#176856',
    paddingHorizontal: 14,
    paddingVertical: 11,
    borderRadius: 8,
    minHeight: 42,
    justifyContent: 'center',
  },
  btnDisabled: { backgroundColor: '#8AA39B' },
  primaryBtnText: { color: '#FFFFFF', fontSize: 13, fontWeight: '700' },
  secondaryBtn: {
    backgroundColor: '#FFFFFF',
    borderWidth: 1,
    borderColor: '#C7D5D0',
    paddingHorizontal: 14,
    paddingVertical: 11,
    borderRadius: 8,
    minHeight: 42,
    justifyContent: 'center',
  },
  secondaryBtnText: { color: '#15362F', fontSize: 13, fontWeight: '600' },
  errorAlert: {
    backgroundColor: '#FCECEE',
    borderWidth: 1,
    borderColor: '#D9828B',
    borderRadius: 8,
    padding: 12,
    gap: 4,
  },
  errorAlertTitle: { color: '#B42332', fontSize: 13, fontWeight: '700' },
  errorAlertBody: { color: '#15362F', fontSize: 12, lineHeight: 18 },
  budgetBox: {
    backgroundColor: '#E7EFEA',
    borderWidth: 1,
    borderColor: '#8AB5A5',
    borderRadius: 10,
    padding: 14,
    gap: 8,
  },
  budgetBoxError: {
    backgroundColor: '#FCECEE',
    borderColor: '#D9828B',
  },
  budgetRow: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    justifyContent: 'space-between',
    alignItems: 'center',
    gap: 8,
  },
  budgetLabel: { color: '#15362F', fontSize: 12, fontWeight: '700' },
  monoPayload: {
    color: '#15362F',
    fontSize: 12,
    backgroundColor: '#FFFFFF',
    padding: 10,
    borderRadius: 6,
    borderWidth: 1,
    borderColor: '#C7D5D0',
  },
  budgetVerdict: { color: '#15362F', fontSize: 12, fontWeight: '700' },
  bandStatusBox: {
    backgroundColor: '#F5F8F7',
    borderRadius: 10,
    borderWidth: 1,
    borderColor: '#C7D5D0',
    padding: 14,
    gap: 10,
  },
  uidBadge: {
    color: '#15362F',
    fontSize: 12,
    fontWeight: '700',
    backgroundColor: '#E7EFEA',
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 6,
  },
  retiredList: { paddingTop: 6, gap: 4 },
  retiredHeading: { color: '#B42332', fontSize: 12, fontWeight: '700' },
  retiredItem: { color: '#4A675D', fontSize: 12 },
});
