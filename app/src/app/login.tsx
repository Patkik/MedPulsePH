import { useState } from 'react';
import { Pressable, StyleSheet, Text, useWindowDimensions, View } from 'react-native';
import { AppShell } from '../components/AppShell';

type StaffRoleKey = 'doctor' | 'reception' | 'band_officer' | 'hospital_admin' | 'privacy_auditor' | 'nurse_disabled';

const HOSPITALS = [
  { id: 'hosp-manila', name: 'Philippine General Synthetic Medical Center (Manila)' },
  { id: 'hosp-cebu', name: 'Vicente Sotto Synthetic Medical Center (Cebu)' },
];

const STAFF_ROLES: Record<
  StaffRoleKey,
  {
    label: string;
    credentialState: string;
    allowedWorkspace: string;
    deniedBoundary: string;
    pilotEnabled: boolean;
  }
> = {
  doctor: {
    label: 'Accredited Doctor',
    credentialState: 'CredentialStatus::Approved (by reviewer cred-rev-1)',
    allowedWorkspace: 'S06–S08 Clinical encounter workspace (requires NFC tap + patient 6-digit PIN)',
    deniedBoundary: 'Cannot self-approve credentials, view other hospitals without sharing consent, or export bulk records.',
    pilotEnabled: true,
  },
  reception: {
    label: 'Reception Staff',
    credentialState: 'CredentialStatus::NotApplicable · Active membership',
    allowedWorkspace: 'S03 Assisted patient registration, identity verification & visit queue',
    deniedBoundary: 'Zero clinical chart access (AuthError::RoleDenied); forbidden from setting or reading a patient PIN.',
    pilotEnabled: true,
  },
  band_officer: {
    label: 'Band Officer',
    credentialState: 'CredentialStatus::NotApplicable · Active membership',
    allowedWorkspace: 'S04 NTAG 424 DNA provisioning, 256-byte NDEF budget check, freeze & replacement',
    deniedBoundary: 'Zero clinical chart access; cannot reactivate a Retired wristband.',
    pilotEnabled: true,
  },
  hospital_admin: {
    label: 'Hospital Administrator',
    credentialState: 'CredentialStatus::NotApplicable · Active membership',
    allowedWorkspace: 'S09 Staff invitations, role scoping & membership suspension',
    deniedBoundary: 'Cannot browse clinical charts or self-approve clinical credentials.',
    pilotEnabled: true,
  },
  privacy_auditor: {
    label: 'Privacy Auditor',
    credentialState: 'CredentialStatus::NotApplicable · Active membership',
    allowedWorkspace: 'S10 Minimized immutable_audit_logs metadata investigation',
    deniedBoundary: 'Receives MinimizedAuditEventProjection only; never decrypted clinical notes or Vault A PII.',
    pilotEnabled: true,
  },
  nurse_disabled: {
    label: 'Nurse / Care-Team (Pilot Disabled)',
    credentialState: 'Deferred in I01 doctor-only pilot scope',
    allowedWorkspace: 'None — returns AuthError::CareTeamClinicalAccessDisabled',
    deniedBoundary: 'All clinical chart reads and writes are blocked in the initial pilot baseline until hospital care-team policy D-06 is approved.',
    pilotEnabled: false,
  },
};

export default function LoginScreen() {
  const [hospitalId, setHospitalId] = useState<string>('hosp-manila');
  const [roleKey, setRoleKey] = useState<StaffRoleKey>('doctor');
  const [mfaVerified, setMfaVerified] = useState<boolean>(true);
  const [contextSwitchCount, setContextSwitchCount] = useState<number>(1);
  const { width } = useWindowDimensions();
  const compact = width > 0 && width < 820;

  const selectedHospital = HOSPITALS.find((h) => h.id === hospitalId) ?? HOSPITALS[0];
  const selectedRole = STAFF_ROLES[roleKey];

  function switchContext(nextHospital: string, nextRole: StaffRoleKey) {
    setHospitalId(nextHospital);
    setRoleKey(nextRole);
    setContextSwitchCount((c) => c + 1);
  }

  return (
    <AppShell
      screenCode="S02 · Staff Sign-In & Context Isolation"
      title="Staff authentication, MFA, and hospital context selection"
      subtitle="Account login, active hospital membership, and independent clinical credential approval are separate checks. Switching hospital or role immediately clears any prior patient context in memory."
      roleBadge={`${selectedRole.label} · ${selectedHospital.id}`}
    >
      <View style={[styles.row, compact && styles.rowCompact]}>
        <View style={styles.card}>
          <Text style={styles.cardTitle}>1. Select active hospital membership</Text>
          <Text style={styles.cardSub}>
            Every protected action is evaluated against the active hospital scope. Cross-hospital chart access without explicit sharing consent returns CrossHospitalAccessDenied.
          </Text>
          <View style={styles.stack}>
            {HOSPITALS.map((h) => {
              const active = h.id === hospitalId;
              return (
                <Pressable
                  key={h.id}
                  accessibilityRole="button"
                  accessibilityState={{ selected: active }}
                  onPress={() => switchContext(h.id, roleKey)}
                  style={[styles.optionBtn, active && styles.optionBtnActive]}
                >
                  <Text style={[styles.optionTitle, active && styles.optionTitleActive]}>
                    {h.name}
                  </Text>
                  <Text style={[styles.optionCode, active && styles.optionCodeActive]}>
                    Scope ID: {h.id}
                  </Text>
                </Pressable>
              );
            })}
          </View>

          <View style={styles.mfaBox}>
            <View style={styles.mfaTextGroup}>
              <Text style={styles.mfaTitle}>Multi-factor authentication (MFA) state</Text>
              <Text style={styles.mfaSub}>
                Missing MFA blocks all staff actions with AuthError::UnauthenticatedOrMissingMfa.
              </Text>
            </View>
            <Pressable
              accessibilityRole="button"
              onPress={() => setMfaVerified((v) => !v)}
              style={[styles.mfaToggle, !mfaVerified && styles.mfaToggleOff]}
            >
              <Text style={styles.mfaToggleText}>
                {mfaVerified ? 'MFA verified' : 'MFA missing (Blocked)'}
              </Text>
            </Pressable>
          </View>
        </View>

        <View style={styles.card}>
          <Text style={styles.cardTitle}>2. Select scoped hospital role</Text>
          <Text style={styles.cardSub}>
            Default-deny RBAC in backend/src/auth.rs grants only the minimum workspace required for each role.
          </Text>
          <View style={styles.roleGrid}>
            {(Object.keys(STAFF_ROLES) as StaffRoleKey[]).map((key) => {
              const active = key === roleKey;
              return (
                <Pressable
                  key={key}
                  accessibilityRole="button"
                  accessibilityState={{ selected: active }}
                  onPress={() => switchContext(hospitalId, key)}
                  style={[styles.roleChip, active && styles.roleChipActive]}
                >
                  <Text style={[styles.roleChipText, active && styles.roleChipTextActive]}>
                    {STAFF_ROLES[key].label}
                  </Text>
                </Pressable>
              );
            })}
          </View>

          <View
            style={[
              styles.decisionPanel,
              (!mfaVerified || !selectedRole.pilotEnabled) && styles.decisionPanelDenied,
            ]}
          >
            <Text style={styles.decisionHeading}>
              {!mfaVerified
                ? 'Access denied · AuthError::UnauthenticatedOrMissingMfa'
                : selectedRole.pilotEnabled
                  ? `Active workspace scope · ${selectedRole.label}`
                  : 'Access denied · AuthError::CareTeamClinicalAccessDisabled'}
            </Text>
            <Text style={styles.decisionItem}>
              Credential check: {selectedRole.credentialState}
            </Text>
            <Text style={styles.decisionItem}>
              Permitted screens: {selectedRole.allowedWorkspace}
            </Text>
            <Text style={styles.decisionItem}>
              Enforced boundary: {selectedRole.deniedBoundary}
            </Text>
            <Text style={styles.resetNote}>
              Context generation #{contextSwitchCount} · Prior patient and encounter state cleared on switch. Tokens are never written to localStorage.
            </Text>
          </View>
        </View>
      </View>
    </AppShell>
  );
}

const styles = StyleSheet.create({
  row: { flexDirection: 'row', alignItems: 'flex-start', gap: 18 },
  rowCompact: { flexDirection: 'column' },
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
  stack: { gap: 10 },
  optionBtn: {
    padding: 14,
    borderRadius: 10,
    borderWidth: 1,
    borderColor: '#C7D5D0',
    backgroundColor: '#F5F8F7',
    gap: 4,
  },
  optionBtnActive: { backgroundColor: '#15362F', borderColor: '#15362F' },
  optionTitle: { color: '#15362F', fontSize: 14, fontWeight: '700' },
  optionTitleActive: { color: '#FFFFFF' },
  optionCode: { color: '#4A675D', fontSize: 12 },
  optionCodeActive: { color: '#A5D2C5' },
  mfaBox: {
    marginTop: 6,
    padding: 14,
    borderRadius: 10,
    backgroundColor: '#E7EFEA',
    borderWidth: 1,
    borderColor: '#C7D5D0',
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    gap: 12,
  },
  mfaTextGroup: { flex: 1, minWidth: 0, gap: 3 },
  mfaTitle: { color: '#15362F', fontSize: 13, fontWeight: '700' },
  mfaSub: { color: '#4A675D', fontSize: 12, lineHeight: 17 },
  mfaToggle: {
    backgroundColor: '#176856',
    paddingHorizontal: 12,
    paddingVertical: 9,
    borderRadius: 8,
  },
  mfaToggleOff: { backgroundColor: '#B42332' },
  mfaToggleText: { color: '#FFFFFF', fontSize: 12, fontWeight: '700' },
  roleGrid: { flexDirection: 'row', flexWrap: 'wrap', gap: 8 },
  roleChip: {
    paddingHorizontal: 13,
    paddingVertical: 10,
    borderRadius: 8,
    borderWidth: 1,
    borderColor: '#C7D5D0',
    backgroundColor: '#F5F8F7',
  },
  roleChipActive: { backgroundColor: '#176856', borderColor: '#176856' },
  roleChipText: { color: '#15362F', fontSize: 13, fontWeight: '600' },
  roleChipTextActive: { color: '#FFFFFF' },
  decisionPanel: {
    padding: 16,
    borderRadius: 10,
    backgroundColor: '#E7EFEA',
    borderWidth: 1,
    borderColor: '#8AB5A5',
    gap: 7,
  },
  decisionPanelDenied: {
    backgroundColor: '#FCECEE',
    borderColor: '#D9828B',
  },
  decisionHeading: { color: '#15362F', fontSize: 15, fontWeight: '700' },
  decisionItem: { color: '#15362F', fontSize: 13, lineHeight: 19 },
  resetNote: { color: '#4A675D', fontSize: 12, marginTop: 4, fontWeight: '600' },
});
