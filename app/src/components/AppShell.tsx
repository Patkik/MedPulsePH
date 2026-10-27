import { Link, usePathname } from 'expo-router';
import { ReactNode } from 'react';
import { Pressable, ScrollView, StyleSheet, Text, useWindowDimensions, View } from 'react-native';

const NAV_LINKS: { href: '/' | '/emergency' | '/patient' | '/doctor' | '/reception' | '/admin' | '/login'; label: string; screenCode: string }[] = [
  { href: '/', label: 'Overview', screenCode: 'Hub' },
  { href: '/emergency', label: 'Emergency tap', screenCode: 'S01' },
  { href: '/login', label: 'Staff sign-in', screenCode: 'S02' },
  { href: '/reception', label: 'Reception & bands', screenCode: 'S03–S04' },
  { href: '/patient', label: 'Patient band', screenCode: 'S05' },
  { href: '/doctor', label: 'Doctor encounter', screenCode: 'S06–S08' },
  { href: '/admin', label: 'Admin & audit', screenCode: 'S09–S10' },
];

export function BrandHeaderMark() {
  return (
    <View style={styles.brand} accessibilityLabel="MedPulsePH">
      <View style={styles.brandMark} accessible={false}>
        <View style={styles.markLeft} />
        <View style={styles.markRight} />
      </View>
      <View>
        <Text style={styles.brandName}>MedPulsePH</Text>
        <Text style={styles.brandSub}>Split-vault NFC clinical architecture</Text>
      </View>
    </View>
  );
}

interface AppShellProps {
  screenCode: string;
  title: string;
  subtitle: string;
  roleBadge: string;
  children: ReactNode;
}

export function AppShell({ screenCode, title, subtitle, roleBadge, children }: AppShellProps) {
  const pathname = usePathname();
  const { width } = useWindowDimensions();
  const compact = width > 0 && width < 860;

  return (
    <ScrollView style={styles.page} contentContainerStyle={styles.pageContent}>
      <View style={[styles.shell, compact && styles.shellCompact]}>
        <View style={[styles.topBar, compact && styles.topBarCompact]}>
          <Link href="/" asChild>
            <Pressable accessibilityRole="link">
              <BrandHeaderMark />
            </Pressable>
          </Link>

          <View style={styles.syntheticPill} accessibilityRole="alert">
            <View style={styles.syntheticDot} />
            <Text style={styles.syntheticText}>
              Synthetic workspace · No live patient records
            </Text>
          </View>
        </View>

        <View style={styles.navBar} accessibilityRole="tablist">
          {NAV_LINKS.map((item) => {
            const active = pathname === item.href;
            return (
              <Link key={item.href} href={item.href} asChild>
                <Pressable
                  accessibilityRole="tab"
                  accessibilityState={{ selected: active }}
                  style={({ pressed }) => [
                    styles.navItem,
                    active && styles.navItemActive,
                    pressed && styles.pressed,
                  ]}
                >
                  <Text style={[styles.navCode, active && styles.navCodeActive]}>
                    {item.screenCode}
                  </Text>
                  <Text style={[styles.navLabel, active && styles.navLabelActive]}>
                    {item.label}
                  </Text>
                </Pressable>
              </Link>
            );
          })}
        </View>

        <View style={styles.pageHeader}>
          <View style={styles.metaRow}>
            <View style={styles.codeBadge}>
              <Text style={styles.codeBadgeText}>{screenCode}</Text>
            </View>
            <View style={styles.roleBadge}>
              <Text style={styles.roleBadgeText}>{roleBadge}</Text>
            </View>
          </View>
          <Text accessibilityRole="header" style={[styles.pageTitle, compact && styles.pageTitleCompact]}>
            {title}
          </Text>
          <Text style={styles.pageSubtitle}>{subtitle}</Text>
        </View>

        <View style={styles.body}>{children}</View>

        <View style={[styles.footer, compact && styles.footerCompact]}>
          <Text style={styles.footerBrand}>MedPulsePH · Zero-trust 13.56 MHz NFC architecture</Text>
          <Text style={styles.footerNote}>
            Vault A (Identity PII) and Vault B (Clinical EHR) remain isolated. Unfinished HTTP clinical routes return 404.
          </Text>
        </View>
      </View>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  page: { flex: 1, backgroundColor: '#F5F8F7' },
  pageContent: { flexGrow: 1 },
  shell: {
    width: '100%',
    maxWidth: 1228,
    alignSelf: 'center',
    paddingHorizontal: 24,
    paddingTop: 18,
    paddingBottom: 36,
    flex: 1,
  },
  shellCompact: { paddingHorizontal: 16 },
  topBar: {
    width: '100%',
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    gap: 16,
    paddingBottom: 16,
  },
  topBarCompact: { flexDirection: 'column', alignItems: 'flex-start', gap: 10 },
  brand: { flexDirection: 'row', alignItems: 'center', gap: 12 },
  brandMark: { width: 32, height: 32, flexDirection: 'row', alignItems: 'center', gap: 4 },
  markLeft: {
    width: 14,
    height: 22,
    borderWidth: 4,
    borderColor: '#15362F',
    borderRightWidth: 0,
    borderTopLeftRadius: 5,
    borderBottomLeftRadius: 5,
  },
  markRight: {
    width: 14,
    height: 14,
    borderWidth: 4,
    borderColor: '#176856',
    borderLeftWidth: 0,
    borderTopRightRadius: 5,
    borderBottomRightRadius: 5,
  },
  brandName: { color: '#15362F', fontSize: 20, fontWeight: '700', letterSpacing: -0.5 },
  brandSub: { color: '#4A675D', fontSize: 12 },
  syntheticPill: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
    backgroundColor: '#E7EFEA',
    borderWidth: 1,
    borderColor: '#C7D5D0',
    paddingHorizontal: 12,
    paddingVertical: 7,
    borderRadius: 8,
  },
  syntheticDot: { width: 8, height: 8, borderRadius: 4, backgroundColor: '#176856' },
  syntheticText: { color: '#15362F', fontSize: 12, fontWeight: '600' },
  navBar: {
    width: '100%',
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
    paddingVertical: 12,
    borderTopWidth: 1,
    borderBottomWidth: 1,
    borderColor: '#C7D5D0',
  },
  navItem: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 7,
    paddingHorizontal: 13,
    paddingVertical: 9,
    borderRadius: 8,
    backgroundColor: '#FFFFFF',
    borderWidth: 1,
    borderColor: '#C7D5D0',
    minHeight: 40,
  },
  navItemActive: { backgroundColor: '#15362F', borderColor: '#15362F' },
  navCode: { color: '#4A675D', fontSize: 11, fontWeight: '700' },
  navCodeActive: { color: '#A5D2C5' },
  navLabel: { color: '#15362F', fontSize: 13, fontWeight: '600' },
  navLabelActive: { color: '#FFFFFF' },
  pressed: { opacity: 0.8 },
  pageHeader: { paddingTop: 24, paddingBottom: 20, gap: 8 },
  metaRow: { flexDirection: 'row', flexWrap: 'wrap', alignItems: 'center', gap: 8 },
  codeBadge: {
    backgroundColor: '#15362F',
    paddingHorizontal: 10,
    paddingVertical: 4,
    borderRadius: 6,
  },
  codeBadgeText: { color: '#FFFFFF', fontSize: 12, fontWeight: '700' },
  roleBadge: {
    backgroundColor: '#E7EFEA',
    borderWidth: 1,
    borderColor: '#C7D5D0',
    paddingHorizontal: 10,
    paddingVertical: 4,
    borderRadius: 6,
  },
  roleBadgeText: { color: '#15362F', fontSize: 12, fontWeight: '600' },
  pageTitle: {
    color: '#15362F',
    fontSize: 34,
    lineHeight: 40,
    fontWeight: '700',
    letterSpacing: -0.9,
    maxWidth: 780,
  },
  pageTitleCompact: { fontSize: 27, lineHeight: 33 },
  pageSubtitle: { color: '#4A675D', fontSize: 15, lineHeight: 23, maxWidth: 720 },
  body: { flex: 1, gap: 20, paddingBottom: 28 },
  footer: {
    width: '100%',
    paddingTop: 20,
    borderTopWidth: 1,
    borderTopColor: '#C7D5D0',
    flexDirection: 'row',
    justifyContent: 'space-between',
    gap: 12,
  },
  footerCompact: { flexDirection: 'column', gap: 6 },
  footerBrand: { color: '#15362F', fontSize: 12, fontWeight: '700' },
  footerNote: { color: '#4A675D', fontSize: 12 },
});
