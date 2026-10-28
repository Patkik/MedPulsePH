import { Link } from 'expo-router';
import { Pressable, ScrollView, StyleSheet, Text, useWindowDimensions, View } from 'react-native';
import { useTheme } from './theme';

export function PublicLayout({ children }: { children: React.ReactNode }) {
  const { width } = useWindowDimensions();
  const { mode, colors, toggle } = useTheme();
  const compact = width < 720;
  return <ScrollView style={[styles.page, { backgroundColor: colors.page }]} contentContainerStyle={styles.grow}>
    <View style={[styles.shell, compact && styles.compact]}>
      <View style={[styles.header, { borderColor: colors.rule }]}>
        <Link href="/" asChild><Pressable accessibilityRole="link" style={styles.brand}><View style={[styles.mark, { backgroundColor: colors.button }]}><Text style={[styles.markText, { color: colors.buttonText }]}>M</Text></View><Text style={[styles.brandText, { color: colors.text }]}>MedPulsePH</Text></Pressable></Link>
        <View style={styles.headerActions}>
          <Pressable accessibilityRole="button" accessibilityLabel={mode === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'} onPress={toggle} style={[styles.headerLink, { borderColor: colors.border }]}><Text style={[styles.headerLinkText, { color: colors.text }]}>{mode === 'dark' ? 'Light mode' : 'Dark mode'}</Text></Pressable>
          <Link href="/login" asChild><Pressable accessibilityRole="link" style={[styles.headerLink, { borderColor: colors.border }]}><Text style={[styles.headerLinkText, { color: colors.text }]}>Sign in</Text></Pressable></Link>
        </View>
      </View>
      <View style={styles.content}>{children}</View>
      <View style={[styles.footer, { borderColor: colors.rule }]}><Text style={[styles.footerText, { color: colors.muted }]}>MedPulsePH</Text><Text style={[styles.footerText, { color: colors.muted }]}>Designed for patients and the people who care for them.</Text></View>
    </View>
  </ScrollView>;
}

export function UnavailablePage({ title, emergency = false }: { title: string; emergency?: boolean }) {
  const { colors } = useTheme();
  return <PublicLayout><View style={styles.notice}><Text style={[styles.eyebrow, { color: colors.accent }]}>Service unavailable</Text><Text accessibilityRole="header" style={[styles.title, { color: colors.text }]}>{title}</Text><Text style={[styles.body, { color: colors.body }]}>This service is not available yet. Do not enter personal or medical information here.</Text>{emergency && <Text style={[styles.emergency, { color: colors.warningText, backgroundColor: colors.warning }]}>If someone needs urgent help, contact local emergency services or seek immediate medical care. Do not wait for this website.</Text>}<Link href="/" asChild><Pressable accessibilityRole="link" style={[styles.primary, { backgroundColor: colors.button }]}><Text style={[styles.primaryText, { color: colors.buttonText }]}>Return to home</Text></Pressable></Link></View></PublicLayout>;
}

export const styles = StyleSheet.create({
  page: { flex: 1 }, grow: { flexGrow: 1 }, shell: { width: '100%', maxWidth: 1180, alignSelf: 'center', paddingHorizontal: 32, flex: 1 }, compact: { paddingHorizontal: 20 },
  header: { minHeight: 88, flexDirection: 'row', alignItems: 'center', justifyContent: 'space-between', borderBottomWidth: 1, gap: 12, flexWrap: 'wrap', paddingVertical: 12 }, headerActions: { flexDirection: 'row', gap: 8, flexWrap: 'wrap' }, brand: { flexDirection: 'row', alignItems: 'center', gap: 10 }, mark: { width: 34, height: 34, borderRadius: 9, alignItems: 'center', justifyContent: 'center' }, markText: { fontSize: 19, fontWeight: '700' }, brandText: { fontSize: 20, fontWeight: '700', letterSpacing: -0.5 }, headerLink: { minHeight: 44, justifyContent: 'center', paddingHorizontal: 12, borderWidth: 1, borderRadius: 7 }, headerLinkText: { fontWeight: '700', fontSize: 14 },
  content: { flex: 1 }, footer: { borderTopWidth: 1, paddingVertical: 24, flexDirection: 'row', flexWrap: 'wrap', justifyContent: 'space-between', gap: 8 }, footerText: { fontSize: 13, lineHeight: 20 }, notice: { maxWidth: 660, paddingVertical: 76, gap: 20, alignItems: 'flex-start' }, eyebrow: { fontWeight: '700', fontSize: 14 }, title: { fontSize: 38, lineHeight: 45, fontWeight: '700', letterSpacing: -1 }, body: { fontSize: 17, lineHeight: 27 }, emergency: { padding: 18, fontSize: 15, lineHeight: 23, borderRadius: 8 }, primary: { borderRadius: 7, minHeight: 48, paddingHorizontal: 20, alignItems: 'center', justifyContent: 'center' }, primaryText: { fontWeight: '700', fontSize: 15 },
});
