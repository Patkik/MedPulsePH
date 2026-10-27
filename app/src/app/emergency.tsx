import { useState } from 'react';
import { Pressable, StyleSheet, Text, useWindowDimensions, View } from 'react-native';
import { AppShell } from '../components/AppShell';

type TapScenario = 'verified_active' | 'frozen_band' | 'replay_rejected' | 'offline_record2';

const SCENARIOS: Record<
  TapScenario,
  {
    label: string;
    statusTitle: string;
    statusTone: 'ok' | 'critical' | 'warn';
    summary: string;
    counterText: string;
    auditEvent: string;
  }
> = {
  verified_active: {
    label: 'Verified NTAG 424 DNA tap (Active)',
    statusTitle: 'Cryptographic SUN proof verified · Band active',
    statusTone: 'ok',
    summary:
      'AES-128 PICCData decrypted and 8-byte truncated AES-CMAC verified in constant time. Counter #14 accepted in Vault A above durable high-water #13.',
    counterText: 'UID 04:91:82:73:64:55:46 · Read counter 14 (> 13)',
    auditEvent: 'emergency.triage_viewed (bound context tap-ctx-014)',
  },
  frozen_band: {
    label: 'Frozen band (403 Locked)',
    statusTitle: 'Wristband is frozen · Emergency summary withheld',
    statusTone: 'critical',
    summary:
      'Vault A reported BandStatus::Frozen before releasing any triage projection. No blood type, allergy, or contact fields are returned.',
    counterText: 'UID 04:91:82:73:64:55:46 · Status: FROZEN (403)',
    auditEvent: 'emergency.tap_denied_band_frozen',
  },
  replay_rejected: {
    label: 'Captured URL replay (Rejected)',
    statusTitle: 'Stale or replayed NFC tap counter rejected',
    statusTone: 'critical',
    summary:
      'Incoming SUN counter #14 is not strictly greater than Vault A durable high-water counter #14 (even if Redis cache was evicted).',
    counterText: 'Incoming counter 14 <= High-water counter 14',
    auditEvent: 'emergency.tap_denied_replay_rejected',
  },
  offline_record2: {
    label: 'Offline NDEF Record 2 fallback',
    statusTitle: 'Offline NDEF Record 2 text · Verify currency when online',
    statusTone: 'warn',
    summary:
      'Read directly from wristband NDEF Record 2 (51 / 120 bytes used; 148 / 256 total NDEF bytes). Offline text cannot confirm live revocation.',
    counterText: 'Record 2 payload: O+|Penicillin,Cefazolin|+639170000123|2026-10-28',
    auditEvent: 'Local reader only (no online server reachability)',
  },
};

export default function EmergencyScreen() {
  const [scenario, setScenario] = useState<TapScenario>('verified_active');
  const { width } = useWindowDimensions();
  const compact = width > 0 && width < 820;
  const activeScenario = SCENARIOS[scenario];
  const showTriageFields = scenario === 'verified_active' || scenario === 'offline_record2';

  return (
    <AppShell
      screenCode="S01 · Tier-1 Public Triage"
      title="Emergency responder wristband view"
      subtitle="Tier-1 public access is strictly limited to blood type, critical allergies, and ICE contact after verifying an NTAG 424 DNA SUN tap and active band status. Clinical history, diagnoses, and medications are never exposed here."
      roleBadge="Anonymous responder · Read-only Tier 1"
    >
      <View style={styles.selectorCard}>
        <Text style={styles.sectionHeading}>Simulate NFC tap verification state</Text>
        <View style={styles.buttonRow}>
          {(Object.keys(SCENARIOS) as TapScenario[]).map((key) => {
            const selected = scenario === key;
            return (
              <Pressable
                key={key}
                accessibilityRole="button"
                accessibilityState={{ selected }}
                onPress={() => setScenario(key)}
                style={[styles.scenarioBtn, selected && styles.scenarioBtnActive]}
              >
                <Text style={[styles.scenarioBtnText, selected && styles.scenarioBtnTextActive]}>
                  {SCENARIOS[key].label}
                </Text>
              </Pressable>
            );
          })}
        </View>
      </View>

      <View
        style={[
          styles.statusBanner,
          activeScenario.statusTone === 'ok' && styles.statusOk,
          activeScenario.statusTone === 'warn' && styles.statusWarn,
          activeScenario.statusTone === 'critical' && styles.statusCritical,
        ]}
        accessibilityRole="alert"
      >
        <Text style={styles.statusTitle}>{activeScenario.statusTitle}</Text>
        <Text style={styles.statusBody}>{activeScenario.summary}</Text>
        <View style={styles.metaStrip}>
          <Text style={styles.metaCode}>{activeScenario.counterText}</Text>
          <Text style={styles.metaCode}>Audit: {activeScenario.auditEvent}</Text>
        </View>
      </View>

      {showTriageFields ? (
        <View style={[styles.grid, compact && styles.gridCompact]}>
          <View style={styles.fieldCard}>
            <View style={styles.fieldHeader}>
              <Text style={styles.fieldLabel}>Blood type</Text>
              <View style={styles.provenanceBadge}>
                <Text style={styles.provenanceText}>
                  {scenario === 'offline_record2'
                    ? 'Offline chip snapshot (2026-10-28)'
                    : 'Clinician verified · PGH Manila'}
                </Text>
              </View>
            </View>
            <Text style={styles.bloodValue}>O RhD positive (O+)</Text>
            <Text style={styles.fieldFootnote}>
              Cross-match remains required before hospital transfusion per clinical protocol.
            </Text>
          </View>

          <View style={[styles.fieldCard, styles.allergyCard]}>
            <View style={styles.fieldHeader}>
              <Text style={styles.fieldLabel}>Critical allergies</Text>
              <View style={styles.provenanceBadge}>
                <Text style={styles.provenanceText}>
                  {scenario === 'offline_record2'
                    ? 'Offline chip snapshot · Never truncated'
                    : 'Clinician verified · High criticality'}
                </Text>
              </View>
            </View>
            <Text style={styles.allergyValue}>
              Penicillin (anaphylaxis), Cefazolin (severe urticaria & bronchospasm)
            </Text>
            <Text style={styles.fieldFootnote}>
              Full allergy text wraps without truncation. Provisioning fails closed if Record 2 exceeds 120 bytes.
            </Text>
          </View>

          <View style={styles.fieldCard}>
            <View style={styles.fieldHeader}>
              <Text style={styles.fieldLabel}>In case of emergency (ICE) contact</Text>
              <View style={styles.provenanceBadge}>
                <Text style={styles.provenanceText}>Patient designated</Text>
              </View>
            </View>
            <Text style={styles.iceValue}>+63 917 000 0123</Text>
            <Text style={styles.fieldFootnote}>
              Emergency notification contact only. Being listed as an ICE contact does not grant clinical record or recovery authority.
            </Text>
          </View>
        </View>
      ) : (
        <View style={styles.lockedCard}>
          <Text style={styles.lockedTitle}>No emergency medical fields disclosed</Text>
          <Text style={styles.lockedBody}>
            Because the wristband proof was rejected ({activeScenario.label}), the API returned a safe 403/400 response without patient identity or triage values. Proceed with standard hospital clinical assessment immediately.
          </Text>
        </View>
      )}
    </AppShell>
  );
}

const styles = StyleSheet.create({
  selectorCard: {
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    borderWidth: 1,
    borderColor: '#C7D5D0',
    padding: 18,
    gap: 12,
  },
  sectionHeading: { color: '#15362F', fontSize: 15, fontWeight: '700' },
  buttonRow: { flexDirection: 'row', flexWrap: 'wrap', gap: 8 },
  scenarioBtn: {
    paddingHorizontal: 14,
    paddingVertical: 10,
    borderRadius: 8,
    borderWidth: 1,
    borderColor: '#C7D5D0',
    backgroundColor: '#F5F8F7',
    minHeight: 42,
    justifyContent: 'center',
  },
  scenarioBtnActive: { backgroundColor: '#176856', borderColor: '#176856' },
  scenarioBtnText: { color: '#15362F', fontSize: 13, fontWeight: '600' },
  scenarioBtnTextActive: { color: '#FFFFFF' },
  statusBanner: {
    borderRadius: 12,
    borderWidth: 1,
    padding: 18,
    gap: 8,
  },
  statusOk: { backgroundColor: '#E7EFEA', borderColor: '#8AB5A5' },
  statusWarn: { backgroundColor: '#FDF6E7', borderColor: '#D6AE5C' },
  statusCritical: { backgroundColor: '#FCECEE', borderColor: '#D9828B' },
  statusTitle: { color: '#15362F', fontSize: 17, fontWeight: '700' },
  statusBody: { color: '#28493F', fontSize: 14, lineHeight: 21 },
  metaStrip: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    justifyContent: 'space-between',
    gap: 8,
    paddingTop: 6,
  },
  metaCode: { color: '#15362F', fontSize: 12, fontWeight: '700' },
  grid: { flexDirection: 'row', flexWrap: 'wrap', gap: 16 },
  gridCompact: { flexDirection: 'column' },
  fieldCard: {
    flex: 1,
    minWidth: 280,
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    borderWidth: 1,
    borderColor: '#C7D5D0',
    padding: 20,
    gap: 10,
  },
  allergyCard: { borderColor: '#B42332', borderWidth: 2 },
  fieldHeader: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    justifyContent: 'space-between',
    alignItems: 'center',
    gap: 8,
  },
  fieldLabel: { color: '#4A675D', fontSize: 13, fontWeight: '700' },
  provenanceBadge: {
    backgroundColor: '#E7EFEA',
    paddingHorizontal: 9,
    paddingVertical: 4,
    borderRadius: 6,
  },
  provenanceText: { color: '#15362F', fontSize: 11, fontWeight: '600' },
  bloodValue: { color: '#15362F', fontSize: 28, fontWeight: '700', letterSpacing: -0.6 },
  allergyValue: { color: '#B42332', fontSize: 20, lineHeight: 27, fontWeight: '700' },
  iceValue: { color: '#15362F', fontSize: 24, fontWeight: '700' },
  fieldFootnote: { color: '#4A675D', fontSize: 12, lineHeight: 18 },
  lockedCard: {
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    borderWidth: 1,
    borderColor: '#D9828B',
    padding: 22,
    gap: 8,
  },
  lockedTitle: { color: '#B42332', fontSize: 18, fontWeight: '700' },
  lockedBody: { color: '#15362F', fontSize: 14, lineHeight: 21 },
});
