import { Link } from 'expo-router';
import { Pressable, StyleSheet, Text, useWindowDimensions, View } from 'react-native';
import { PublicLayout, styles as shared } from '../components/PublicLayout';

export default function HomeScreen() {
  const { width } = useWindowDimensions();
  const compact = width < 720;
  return <PublicLayout>
    <View style={[styles.hero, compact && styles.heroCompact]}>
      <View style={styles.intro}>
        <Text style={styles.kicker}>For patients and care teams</Text>
        <Text accessibilityRole="header" style={[styles.heading, compact && styles.headingCompact]}>Care information, closer when it matters.</Text>
        <Text style={styles.lede}>MedPulsePH is being built to help patients and hospitals share essential information with care and clarity.</Text>
        <Link href="/login" asChild><Pressable accessibilityRole="link" style={shared.primary}><Text style={shared.primaryText}>Go to sign in</Text></Pressable></Link>
      </View>
      <View style={styles.side}><Text style={styles.sideLabel}>A note on availability</Text><Text style={styles.sideHeading}>The service is not open yet.</Text><Text style={styles.sideBody}>Accounts, wristband access, and clinical records are still being developed. Please do not enter patient information.</Text></View>
    </View>
    <View style={styles.sections}>
      <View style={styles.section}><Text style={styles.sectionTitle}>For patients</Text><Text style={styles.sectionBody}>A planned place to manage wristband access and understand how your information is shared.</Text></View>
      <View style={styles.section}><Text style={styles.sectionTitle}>For hospitals</Text><Text style={styles.sectionBody}>Planned tools for authorized staff to support registration and patient care.</Text></View>
      <View style={styles.section}><Text style={styles.sectionTitle}>In an emergency</Text><Text style={styles.sectionBody}>This website cannot provide emergency information yet. Contact local emergency services or seek immediate medical care.</Text></View>
    </View>
  </PublicLayout>;
}
const styles = StyleSheet.create({
  hero: { paddingVertical: 80, flexDirection: 'row', alignItems: 'center', gap: 56 }, heroCompact: { flexDirection: 'column', alignItems: 'stretch', paddingVertical: 48, gap: 32 }, intro: { flex: 1, alignItems: 'flex-start', gap: 20 }, kicker: { fontSize: 14, color: '#176856', fontWeight: '700' }, heading: { fontSize: 52, lineHeight: 59, color: '#15362F', fontWeight: '700', letterSpacing: -1.8, maxWidth: 650 }, headingCompact: { fontSize: 38, lineHeight: 45, letterSpacing: -1 }, lede: { color: '#425E55', fontSize: 18, lineHeight: 28, maxWidth: 570 }, side: { width: '100%', maxWidth: 360, backgroundColor: '#E3EEEA', padding: 28, borderRadius: 10, gap: 12 }, sideLabel: { color: '#37695B', fontSize: 13, fontWeight: '700' }, sideHeading: { color: '#15362F', fontSize: 24, lineHeight: 30, fontWeight: '700' }, sideBody: { color: '#425E55', fontSize: 15, lineHeight: 23 }, sections: { flexDirection: 'row', flexWrap: 'wrap', gap: 28, paddingBottom: 72 }, section: { flex: 1, minWidth: 220, borderTopWidth: 2, borderColor: '#A8C6BB', paddingTop: 20, gap: 10 }, sectionTitle: { color: '#15362F', fontSize: 19, fontWeight: '700' }, sectionBody: { color: '#526C63', fontSize: 15, lineHeight: 24 },
});
