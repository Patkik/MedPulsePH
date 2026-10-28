import { createContext, ReactNode, useContext, useEffect, useState } from 'react';
import { Platform, useColorScheme } from 'react-native';

type Mode = 'light' | 'dark';
const key = 'medpulseph.theme';
const palettes = {
  light: { page: '#F5F8F7', text: '#15362F', body: '#425E55', muted: '#526C63', accent: '#176856', border: '#9BB7AD', rule: '#D5E1DD', panel: '#E3EEEA', panelLabel: '#37695B', sectionRule: '#A8C6BB', button: '#153F36', buttonText: '#FFFFFF', warning: '#FFF3DF', warningText: '#6F3815' },
  dark: { page: '#132621', text: '#EDF7F2', body: '#C5DAD1', muted: '#BAD0C6', accent: '#A8DCC8', border: '#77988B', rule: '#39584C', panel: '#203D33', panelLabel: '#B5E1CF', sectionRule: '#6C9785', button: '#B8E2D1', buttonText: '#102A21', warning: '#4C361E', warningText: '#FFE1B5' },
};
const ThemeContext = createContext<{ mode: Mode; colors: typeof palettes.light; toggle: () => void } | null>(null);

export function ThemeProvider({ children }: { children: ReactNode }) {
  const system = useColorScheme();
  const [choice, setChoice] = useState<Mode | null>(null);
  useEffect(() => {
    if (Platform.OS !== 'web') return;
    try {
      const saved = window.localStorage.getItem(key);
      // Restore browser-only storage after static rendering to avoid a hydration mismatch.
      // eslint-disable-next-line react-hooks/set-state-in-effect
      if (saved === 'light' || saved === 'dark') setChoice(saved);
    } catch { /* Storage may be unavailable; the theme still works for this session. */ }
  }, []);
  const mode = choice ?? (system === 'dark' ? 'dark' : 'light');
  function toggle() {
    const next = mode === 'dark' ? 'light' : 'dark';
    setChoice(next);
    if (Platform.OS === 'web') {
      try { window.localStorage.setItem(key, next); } catch { /* Keep the in-memory choice. */ }
    }
  }
  return <ThemeContext.Provider value={{ mode, colors: palettes[mode], toggle }}>{children}</ThemeContext.Provider>;
}

export function useTheme() {
  const theme = useContext(ThemeContext);
  if (!theme) throw new Error('ThemeProvider is required');
  return theme;
}
