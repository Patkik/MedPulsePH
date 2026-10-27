import { Link } from 'expo-router';
import { useState } from 'react';
import { Pressable, StyleSheet, Text, useWindowDimensions, View } from 'react-native';
import { AppShell } from '../components/AppShell';

type RoleKey = 'emergency' | 'patient' | 'doctor' | 'reception' | 'admin';

const ROLES: Record<
  RoleKey,
  {
    label: string;
    screenCode: string;
    route: '/emergency' | '/patient' | '/doctor' | '/reception' | '/admin';
    heading: string;
    description: string;
    capabilities: string[];
    guardrail: string;
  }
> = {
  emergency: {
    label: 'Emergency responder',
    screenCode: 'S01',
    route: '/emergency',
    heading: 'Permitted emergency summary from an active wristband tap.',
    description:
      'Tier-1 public access verifies the NTAG 424 DNA SUN cryptographic counter and active band status, returning only blood type, critical allergies, and ICE contact.',
    capabilities: [
      'Verify AES-128 PICCData counter & 8-byte truncated AES-CMAC',
      'Display blood type, critical allergies, and ICE contact with provenance',
      'Fallback to NDEF Record 2 offline text with staleness notice',
    ],
    guardrail: 'Never exposes past encounters, diagnoses, medications, or frozen bands (403).',
  },
  patient: {
    label: 'Patient',
    screenCode: 'S05',
    route: '/patient',
    heading: 'Your wristband, clinical consent requests, and access log.',
    description:
      'Freeze or unfreeze your wristband immediately, approve or decline 15-minute clinical encounter requests with your private 6-digit PIN, and inspect every recorded access.',
    capabilities: [
      'One-tap wristband freeze & unfreeze (invalidates active sessions)',
      'Approve or decline 15-minute doctor requests without blame',
      'Control cross-hospital longitudinal sharing & view immutable access history',
    ],
    guardrail: 'Freezing a band stops online access immediately without deleting medical records.',
  },
  doctor: {
    label: 'Accredited doctor',
    screenCode: 'S06–S08',
    route: '/doctor',
    heading: 'Care starts with verified tap proof and patient PIN approval.',
    description:
      'Pair a PC workstation, accept a verified NTAG 424 DNA tap, complete the patient 6-digit PIN handshake to mint a 15-minute VerifiedClinicSession, and document encounters.',
    capabilities: [
      'Bind short-lived PC workstation pairing challenge to verified NFC tap',
      'Mint fixed 15-minute VerifiedClinicSession via Argon2id PIN handshake',
      'Author Draft -> Reviewed -> Finalized -> Amended AES-256-GCM encounters',
    ],
    guardrail: 'Transaction boundary revalidates expiry, band status, and audit storage health on every read/write.',
  },
  reception: {
    label: 'Reception & Band Officer',
    screenCode: 'S03–S04',
    route: '/reception',
    heading: 'Assisted identity verification and 256-byte NTAG 424 DNA provisioning.',
    description:
      'Verify patient identity links with explicit evidence, hand off private PIN entry to the patient, check the 256-byte NDEF dual-record budget, and atomically replace bands.',
    capabilities: [
      'Advance PatientIdentityLinkState (Proposed -> EvidencePending -> Verified)',
      'Enforce 256-byte NTAG 424 DNA NDEF budget (refusing allergy truncation)',
      'Issue replacement wristbands and permanently retire old band UIDs',
    ],
    guardrail: 'Staff roles are forbidden from setting or reading a patient’s 6-digit PIN or opening clinical charts.',
  },
  admin: {
    label: 'Hospital Admin & Privacy Auditor',
    screenCode: 'S09–S10',
    route: '/admin',
    heading: 'Scoped governance, independent credential review, and minimized audit.',
    description:
      'Manage hospital memberships, require independent reviewer sign-off for clinical eligibility, and investigate minimized immutable_audit_logs projections.',
    capabilities: [
      'Block self-approval of clinical credentials (AuthError::SelfApprovalForbidden)',
      'Suspend clinician membership and increment revocation version immediately',
      'Inspect MinimizedAuditEventProjection without clinical note plaintexts',
    ],
    guardrail: 'Administrators and privacy auditors never receive decrypted Vault B clinical notes.',
  },
};

const ALL_PAGES: {
  code: string;
  title: string;
  route: '/emergency' | '/login' | '/reception' | '/patient' | '/doctor' | '/admin';
  summary: string;
}[] = [
  {
    code: 'S01',
    title: 'Emergency responder triage view',
    route: '/emergency',
    summary: 'Verified NTAG 424 DNA tap, frozen-band 403 lock, replay rejection, and offline NDEF Record 2 fallback.',
  },
  {
    code: 'S02',
    title: 'Staff sign-in, MFA & hospital switcher',
    route: '/login',
    summary: 'Hospital scope isolation, MFA enforcement, role boundary preview, and disabled care-team gate.',
  },
  {
    code: 'S03 & S04',
    title: 'Reception enrollment & Band Officer NDEF budget',
    route: '/reception',
    summary: 'Identity link verification, private PIN keypad boundary, 256-byte NDEF calculator, and band replacement.',
  },
  {
    code: 'S05',
    title: 'Patient wristband, consent & access history',
    route: '/patient',
    summary: 'One-tap band freeze, 15-minute PIN request approval/decline, cross-hospital sharing, and audit log.',
  },
  {
    code: 'S06–S08',
    title: 'Doctor pairing, PIN handshake & clinical encounter',
    route: '/doctor',
    summary: 'Workstation pairing, 6-digit PIN handshake, 15:00 countdown, Draft -> Finalized -> Amended revisions, and fault checks.',
  },
  {
    code: 'S09 & S10',
    title: 'Hospital Admin, Credential Review & Privacy Audit',
    route: '/admin',
    summary: 'Self-approval block, clinician suspension, minimized privacy auditor projection, and bulk-export denial.',
  },
];

export default function HomeScreen() {
  const [role, setRole] = useState<RoleKey>('patient');
  const { width } = useWindowDimensions();
  const compact = width > 0 && width < 860;
  const selected = ROLES[role];

  return (
    <AppShell
      screenCode="S01–S10 · Interactive Workspace Hub"
      title={selected.heading}
      subtitle={selected.description}
      roleBadge={`Selected lens: ${selected.label} (${selected.screenCode})`}
    >
      <View style={[styles.mainRow, compact && styles.mainRowCompact]}>
        <View style={styles.leftCol}>
          <Text style={styles.sectionTitle}>Select a role lens</Text>
          <View style={styles.roleGroup} accessibilityLabel="Select a role lens">
            {(Object.keys(ROLES) as RoleKey[]).map((key) => {
              const active = role === key;
              return (
                <Pressable
                  key={key}
                  accessibilityRole="button"
                  accessibilityState={{ selected: active }}
                  onPress={() => setRole(key)}
                  style={[styles.roleButton, active && styles.roleButtonActive]}
                >
                  <Text style={[styles.roleButtonText, active && styles.roleButtonTextActive]}>
                    {ROLES[key].label}
                  </Text>
                </Pressable>
              );
            })}
          </View>

          <View style={styles.guardrailBox}>
            <Text style={styles.guardrailTitle}>Enforced security boundary ({selected.screenCode})</Text>
            <Text style={styles.guardrailBody}>{selected.guardrail}</Text>
          </View>
        </View>

        <View style={styles.panel}>
          <View style={styles.panelTop}>
            <View style={styles.panelTitleWrap}>
              <Text style={styles.panelCode}> Screen {selected.screenCode}</Text>
              <Text style={styles.panelTitle}>{selected.label}</Text>
            </View>
            <Link href={selected.route} asChild>
              <Pressable accessibilityRole="link" style={styles.openBtn}>
                <Text style={styles.openBtnText}>Open interactive page</Text>
              </Pressable>
            </Link>
          </View>

          <View style={styles.capList}>
            {selected.capabilities.map((cap) => (
              <View key={cap} style={styles.capRow}>
                <View style={styles.capDot} />
                <Text style={styles.capText}>{cap}</Text>
              </View>
            ))}
          </View>
        </View>
      </View>

      <View style={styles.catalogSection}>
        <Text style={styles.sectionTitle}>All interactive pages (S01–S10 screen catalog)</Text>
        <View style={[styles.catalogGrid, compact && styles.catalogGridCompact]}>
          {ALL_PAGES.map((page) => (
            <View key={page.code} style={styles.catalogCard}>
              <View style={styles.catalogTop}>
                <Text style={styles.catalogCode}>{page.code}</Text>
                <Link href={page.route} asChild>
                  <Pressable accessibilityRole="link" style={styles.catalogLinkBtn}>
                    <Text style={styles.catalogLinkText}>Open {page.route}</Text>
                  </Pressable>
                </Link>
              </View>
              <Text style={styles.catalogCardTitle}>{page.title}</Text>
              <Text style={styles.catalogCardBody}>{page.summary}</Text>
            </View>
          ))}
        </View>
      </View>
    </AppShell>
  );
}

const styles = StyleSheet.create({
  mainRow: { flexDirection: 'row', alignItems: 'stretch', gap: 18 },
  mainRowCompact: { flexDirection: 'column' },
  leftCol: {
    flex: 1,
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    borderWidth: 1,
    borderColor: '#C7D5D0',
    padding: 20,
    gap: 14,
  },
  sectionTitle: { color: '#15362F', fontSize: 17, fontWeight: '700' },
  roleGroup: { flexDirection: 'row', flexWrap: 'wrap', gap: 8 },
  roleButton: {
    paddingHorizontal: 14,
    paddingVertical: 10,
    borderRadius: 8,
    borderWidth: 1,
    borderColor: '#C7D5D0',
    backgroundColor: '#F5F8F7',
    minHeight: 42,
    justifyContent: 'center',
  },
  roleButtonActive: { backgroundColor: '#176856', borderColor: '#176856' },
  roleButtonText: { color: '#15362F', fontSize: 13, fontWeight: '600' },
  roleButtonTextActive: { color: '#FFFFFF' },
  guardrailBox: {
    backgroundColor: '#E7EFEA',
    borderRadius: 10,
    borderWidth: 1,
    borderColor: '#8AB5A5',
    padding: 14,
    gap: 4,
    marginTop: 4,
  },
  guardrailTitle: { color: '#15362F', fontSize: 13, fontWeight: '700' },
  guardrailBody: { color: '#15362F', fontSize: 13, lineHeight: 19 },
  panel: {
    flex: 1,
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    borderWidth: 1,
    borderColor: '#C7D5D0',
    padding: 20,
    gap: 14,
  },
  panelTop: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    justifyContent: 'space-between',
    alignItems: 'center',
    gap: 10,
  },
  panelTitleWrap: { flex: 1, minWidth: 180, gap: 2 },
  panelCode: { color: '#4A675D', fontSize: 12, fontWeight: '700' },
  panelTitle: { color: '#15362F', fontSize: 22, fontWeight: '700', letterSpacing: -0.5 },
  openBtn: {
    backgroundColor: '#15362F',
    paddingHorizontal: 14,
    paddingVertical: 10,
    borderRadius: 8,
  },
  openBtnText: { color: '#FFFFFF', fontSize: 13, fontWeight: '700' },
  capList: { gap: 10, paddingTop: 4 },
  capRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 10,
    paddingTop: 10,
    borderTopWidth: 1,
    borderTopColor: '#E7EFEA',
  },
  capDot: { width: 7, height: 7, borderRadius: 4, backgroundColor: '#176856' },
  capText: { flex: 1, color: '#15362F', fontSize: 13, lineHeight: 19, fontWeight: '600' },
  catalogSection: { gap: 12 },
  catalogGrid: { flexDirection: 'row', flexWrap: 'wrap', gap: 14 },
  catalogGridCompact: { flexDirection: 'column' },
  catalogCard: {
    flex: 1,
    minWidth: 320,
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    borderWidth: 1,
    borderColor: '#C7D5D0',
    padding: 18,
    gap: 8,
  },
  catalogTop: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    gap: 8,
  },
  catalogCode: {
    backgroundColor: '#E7EFEA',
    color: '#15362F',
    fontSize: 12,
    fontWeight: '700',
    paddingHorizontal: 9,
    paddingVertical: 4,
    borderRadius: 6,
  },
  catalogLinkBtn: {
    backgroundColor: '#176856',
    paddingHorizontal: 12,
    paddingVertical: 7,
    borderRadius: 6,
  },
  catalogLinkText: { color: '#FFFFFF', fontSize: 12, fontWeight: '700' },
  catalogCardTitle: { color: '#15362F', fontSize: 16, fontWeight: '700' },
  catalogCardBody: { color: '#4A675D', fontSize: 13, lineHeight: 19 },
});
