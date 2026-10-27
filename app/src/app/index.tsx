import { useState } from 'react';
import { Pressable, ScrollView, StyleSheet, Text, useWindowDimensions, View } from 'react-native';

type Role = 'patient' | 'doctor' | 'hospital';

const roles: Record<Role, { label: string; heading: string; description: string; actions: string[] }> = {
  patient: {
    label: 'Patient',
    heading: 'Your health, in your hands.',
    description: 'Manage your wristband, review requests for access, and stay informed about who has viewed your information.',
    actions: ['Manage your wristband', 'Review access requests', 'See your access history'],
  },
  doctor: {
    label: 'Doctor',
    heading: 'Care starts with the right context.',
    description: 'Bring patient authorization and clinical information together in one focused encounter workspace.',
    actions: ['Request patient authorization', 'Review a permitted chart', 'Document an encounter'],
  },
  hospital: {
    label: 'Hospital admin',
    heading: 'A clearer view of your hospital.',
    description: 'Manage staff access and operational activity with permissions tailored to each responsibility.',
    actions: ['Manage staff access', 'Review credential status', 'Monitor service activity'],
  },
};

function Brand() {
  return (
    <View style={styles.brand} accessibilityLabel="MedPulsePH">
      <View style={styles.brandMark} accessible={false}>
        <View style={styles.markLeft} />
        <View style={styles.markRight} />
      </View>
      <Text style={styles.brandName}>MedPulsePH</Text>
    </View>
  );
}

export default function HomeScreen() {
  const [role, setRole] = useState<Role>('patient');
  const { width } = useWindowDimensions();
  const compact = width < 760;
  const selected = roles[role];

  return (
    <ScrollView style={styles.page} contentContainerStyle={styles.pageContent}>
      <View style={[styles.header, compact && styles.headerCompact]}>
        <Brand />
        <View style={styles.headerMeta}>
          <View style={styles.previewDot} />
          <Text style={styles.previewLabel}>DESIGN PREVIEW</Text>
        </View>
      </View>

      <View style={styles.notice} accessibilityRole="alert">
        <Text style={styles.noticeTitle}>A preview with no patient data</Text>
        <Text style={styles.noticeText}>Sign-in, wristband scanning, and clinical records are not connected yet.</Text>
      </View>

      <View style={[styles.main, compact && styles.mainCompact]}>
        <View style={styles.intro}>
          <Text style={styles.eyebrow}>CARE, CONNECTED</Text>
          <Text accessibilityRole="header" style={[styles.hero, compact && styles.heroCompact]}>{selected.heading}</Text>
          <Text style={styles.description}>{selected.description}</Text>

          <Text style={styles.selectorLabel}>Explore the experience for</Text>
          <View style={styles.roleGroup} accessibilityLabel="Preview a role">
            {(Object.keys(roles) as Role[]).map((item) => (
              <Pressable
                key={item}
                accessibilityRole="button"
                accessibilityState={{ selected: role === item }}
                onPress={() => setRole(item)}
                style={({ pressed }) => [styles.roleButton, role === item && styles.roleButtonActive, pressed && styles.pressed]}
              >
                <Text style={[styles.roleButtonText, role === item && styles.roleButtonTextActive]}>{roles[item].label}</Text>
              </Pressable>
            ))}
          </View>
          <Text style={styles.selectorHint}>Choose a role to see its planned workspace.</Text>
        </View>

        <View style={styles.panel}>
          <View style={styles.panelTop}>
            <View>
              <Text style={styles.panelKicker}>WORKSPACE PREVIEW</Text>
              <Text style={styles.panelTitle}>{selected.label}</Text>
            </View>
            <View style={styles.panelBadge}><Text style={styles.panelBadgeText}>COMING SOON</Text></View>
          </View>
          <Text style={styles.panelIntro}>Designed around the things that matter most.</Text>
          {selected.actions.map((action, index) => (
            <View key={action} style={styles.actionRow}>
              <Text style={styles.actionNumber}>0{index + 1}</Text>
              <Text style={styles.actionText}>{action}</Text>
              <Text style={styles.actionStatus}>Planned</Text>
            </View>
          ))}
          <View style={styles.panelFooter}>
            <Text style={styles.panelFooterTitle}>Privacy is part of the design.</Text>
            <Text style={styles.panelFooterText}>Access to clinical records will require an authorized clinician and patient approval.</Text>
          </View>
        </View>
      </View>

      <View style={[styles.footer, compact && styles.footerCompact]}>
        <Text style={styles.footerBrand}>MedPulsePH</Text>
        <Text style={styles.footerText}>A more thoughtful way to connect care.</Text>
        <Text style={styles.footerText}>Built for people, with privacy in mind.</Text>
      </View>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  page: { flex: 1, backgroundColor: '#F3F7F5' },
  pageContent: { minHeight: '100%', paddingHorizontal: 24 },
  header: { width: '100%', maxWidth: 1180, alignSelf: 'center', height: 96, flexDirection: 'row', alignItems: 'center', justifyContent: 'space-between' },
  headerCompact: { height: 78 },
  brand: { flexDirection: 'row', alignItems: 'center', gap: 10 },
  brandMark: { width: 32, height: 32, flexDirection: 'row', alignItems: 'center', gap: 4 },
  markLeft: { width: 14, height: 22, borderWidth: 4, borderColor: '#1B5146', borderRightWidth: 0, borderTopLeftRadius: 5, borderBottomLeftRadius: 5 },
  markRight: { width: 14, height: 14, borderWidth: 4, borderColor: '#1B5146', borderLeftWidth: 0, borderTopRightRadius: 5, borderBottomRightRadius: 5 },
  brandName: { color: '#173B33', fontSize: 21, fontWeight: '700', letterSpacing: -0.7 },
  headerMeta: { flexDirection: 'row', alignItems: 'center', gap: 7 },
  previewDot: { width: 7, height: 7, borderRadius: 4, backgroundColor: '#7AA78E' },
  previewLabel: { fontSize: 10, fontWeight: '700', letterSpacing: 1.1, color: '#687F75' },
  notice: { width: '100%', maxWidth: 1180, alignSelf: 'center', backgroundColor: '#E5EEE8', borderRadius: 10, paddingHorizontal: 18, paddingVertical: 12, gap: 3 },
  noticeTitle: { color: '#244E3E', fontSize: 13, fontWeight: '700' },
  noticeText: { color: '#496B5C', fontSize: 13, lineHeight: 19 },
  main: { width: '100%', maxWidth: 1180, alignSelf: 'center', flex: 1, flexDirection: 'row', alignItems: 'center', gap: 72, paddingVertical: 86 },
  mainCompact: { flexDirection: 'column', alignItems: 'stretch', gap: 40, paddingVertical: 48 },
  intro: { flex: 1, maxWidth: 570 },
  eyebrow: { fontSize: 11, fontWeight: '700', letterSpacing: 1.8, color: '#628274', marginBottom: 20 },
  hero: { fontSize: 56, lineHeight: 62, letterSpacing: -2.2, fontWeight: '700', color: '#173B33', maxWidth: 540 },
  heroCompact: { fontSize: 40, lineHeight: 46, letterSpacing: -1.4 },
  description: { fontSize: 17, lineHeight: 27, color: '#557267', maxWidth: 490, marginTop: 20 },
  selectorLabel: { fontSize: 14, color: '#294D40', fontWeight: '700', marginTop: 38, marginBottom: 12 },
  roleGroup: { flexDirection: 'row', flexWrap: 'wrap', gap: 8 },
  roleButton: { paddingHorizontal: 16, paddingVertical: 12, borderRadius: 8, borderWidth: 1, borderColor: '#C8D8D0', backgroundColor: '#F8FBF9', minHeight: 44, justifyContent: 'center' },
  roleButtonActive: { backgroundColor: '#245A49', borderColor: '#245A49' },
  roleButtonText: { fontSize: 13, color: '#385E50', fontWeight: '600' },
  roleButtonTextActive: { color: '#FFFFFF' },
  pressed: { opacity: 0.75 },
  selectorHint: { color: '#789084', fontSize: 12, marginTop: 12 },
  panel: { flex: 1, width: '100%', maxWidth: 500, borderRadius: 16, backgroundColor: '#FFFFFF', borderWidth: 1, borderColor: '#DDE8E1', padding: 26 },
  panelTop: { flexDirection: 'row', alignItems: 'flex-start', justifyContent: 'space-between', gap: 10 },
  panelKicker: { color: '#849A8E', fontSize: 10, letterSpacing: 1.2, fontWeight: '700', marginBottom: 8 },
  panelTitle: { color: '#244A3D', fontSize: 23, fontWeight: '700', letterSpacing: -0.6 },
  panelBadge: { borderRadius: 20, backgroundColor: '#EDF4EF', paddingHorizontal: 10, paddingVertical: 7 },
  panelBadgeText: { fontSize: 9, color: '#658676', fontWeight: '700', letterSpacing: 0.6 },
  panelIntro: { color: '#71897D', fontSize: 13, lineHeight: 20, marginTop: 13, marginBottom: 20 },
  actionRow: { minHeight: 62, borderTopWidth: 1, borderTopColor: '#EDF1EE', flexDirection: 'row', alignItems: 'center', gap: 14 },
  actionNumber: { color: '#9AAEA2', fontSize: 11, fontWeight: '700' },
  actionText: { flex: 1, color: '#35594A', fontSize: 14, fontWeight: '600' },
  actionStatus: { color: '#859B8E', fontSize: 12 },
  panelFooter: { backgroundColor: '#F1F6F2', borderRadius: 10, padding: 16, marginTop: 16, gap: 6 },
  panelFooterTitle: { color: '#315A48', fontSize: 13, fontWeight: '700' },
  panelFooterText: { color: '#6D897B', fontSize: 12, lineHeight: 19 },
  footer: { width: '100%', maxWidth: 1180, alignSelf: 'center', paddingVertical: 28, borderTopWidth: 1, borderTopColor: '#DEE9E2', flexDirection: 'row', justifyContent: 'space-between', gap: 10 },
  footerCompact: { flexDirection: 'column' },
  footerBrand: { color: '#315749', fontSize: 12, fontWeight: '700' },
  footerText: { color: '#849A8E', fontSize: 12 },
});
