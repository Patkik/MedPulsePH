import { Link } from 'expo-router';
import { Pressable, ScrollView, StyleSheet, Text, useWindowDimensions, View } from 'react-native';

export function PublicLayout({ children }: { children: React.ReactNode }) {
  const { width } = useWindowDimensions();
  const compact = width < 720;
  return <ScrollView style={styles.page} contentContainerStyle={styles.grow}>
    <View style={[styles.shell, compact && styles.compact]}>
      <View style={styles.header}>
        <Link href="/" asChild><Pressable accessibilityRole="link" style={styles.brand}><View style={styles.mark}><Text style={styles.markText}>M</Text></View><Text style={styles.brandText}>MedPulsePH</Text></Pressable></Link>
        <Link href="/login" asChild><Pressable accessibilityRole="link" style={styles.headerLink}><Text style={styles.headerLinkText}>Sign in</Text></Pressable></Link>
      </View>
      <View style={styles.content}>{children}</View>
      <View style={styles.footer}><Text style={styles.footerText}>MedPulsePH</Text><Text style={styles.footerText}>Designed for patients and the people who care for them.</Text></View>
    </View>
  </ScrollView>;
}

export function UnavailablePage({ title, emergency = false }: { title: string; emergency?: boolean }) {
  return <PublicLayout><View style={styles.notice}><Text style={styles.eyebrow}>Service unavailable</Text><Text accessibilityRole="header" style={styles.title}>{title}</Text><Text style={styles.body}>This service is not available yet. Do not enter personal or medical information here.</Text>{emergency && <Text style={styles.emergency}>If someone needs urgent help, contact local emergency services or seek immediate medical care. Do not wait for this website.</Text>}<Link href="/" asChild><Pressable accessibilityRole="link" style={styles.primary}><Text style={styles.primaryText}>Return to home</Text></Pressable></Link></View></PublicLayout>;
}

export const styles = StyleSheet.create({
  page: { flex: 1, backgroundColor: '#F5F8F7' }, grow: { flexGrow: 1 }, shell: { width: '100%', maxWidth: 1180, alignSelf: 'center', paddingHorizontal: 32, flex: 1 }, compact: { paddingHorizontal: 20 },
  header: { minHeight: 88, flexDirection: 'row', alignItems: 'center', justifyContent: 'space-between', borderBottomWidth: 1, borderColor: '#D5E1DD', gap: 12 }, brand: { flexDirection: 'row', alignItems: 'center', gap: 10 }, mark: { width: 34, height: 34, borderRadius: 9, backgroundColor: '#153F36', alignItems: 'center', justifyContent: 'center' }, markText: { color: '#fff', fontSize: 19, fontWeight: '700' }, brandText: { color: '#15362F', fontSize: 20, fontWeight: '700', letterSpacing: -0.5 }, headerLink: { minHeight: 44, justifyContent: 'center', paddingHorizontal: 16, borderWidth: 1, borderColor: '#9BB7AD', borderRadius: 7 }, headerLinkText: { color: '#153F36', fontWeight: '700', fontSize: 14 },
  content: { flex: 1 }, footer: { borderTopWidth: 1, borderColor: '#D5E1DD', paddingVertical: 24, flexDirection: 'row', flexWrap: 'wrap', justifyContent: 'space-between', gap: 8 }, footerText: { color: '#526C63', fontSize: 13, lineHeight: 20 }, notice: { maxWidth: 660, paddingVertical: 76, gap: 20, alignItems: 'flex-start' }, eyebrow: { color: '#176856', fontWeight: '700', fontSize: 14 }, title: { color: '#15362F', fontSize: 38, lineHeight: 45, fontWeight: '700', letterSpacing: -1 }, body: { color: '#425E55', fontSize: 17, lineHeight: 27 }, emergency: { color: '#6F3815', backgroundColor: '#FFF3DF', padding: 18, fontSize: 15, lineHeight: 23, borderRadius: 8 }, primary: { backgroundColor: '#153F36', borderRadius: 7, minHeight: 48, paddingHorizontal: 20, alignItems: 'center', justifyContent: 'center' }, primaryText: { color: '#fff', fontWeight: '700', fontSize: 15 },
});
