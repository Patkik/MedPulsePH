import { Stack } from 'expo-router';
import { ThemeProvider } from '../components/theme';

export default function RootLayout() {
  return <ThemeProvider><Stack screenOptions={{ headerShown: false }} /></ThemeProvider>;
}
