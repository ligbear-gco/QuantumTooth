/** @file @brief One portrait proof screen; only native verified events populate the result. */
import React, {useEffect, useRef, useState} from 'react';
import {
  AppState, Modal, PermissionsAndroid, Platform, Pressable, ScrollView,
  StatusBar, StyleSheet, Text, View,
} from 'react-native';
import {SafeAreaProvider, SafeAreaView} from 'react-native-safe-area-context';
import QTooth, {type ProofEvent} from './specs/NativeQTooth';

/** @brief Status history is capped; plaintext is never persisted or copied into logs. */
function Screen(): React.JSX.Element {
  const [phase, setPhase] = useState('Idle · start a Pi send, then scan');
  const [devices, setDevices] = useState<string[]>([]);
  const [selected, setSelected] = useState('');
  const [result, setResult] = useState<ProofEvent | null>(null);
  const [details, setDetails] = useState(false);
  const [history, setHistory] = useState<string[]>([]);
  const [busy, setBusy] = useState(false);
  const generation = useRef(-1);
  const accepting = useRef(false);

  useEffect(() => {
    const events = QTooth.onEvent(event => {
      if (event.kind === 'attempt') {
        generation.current = event.attempt;
        accepting.current = true;
        setResult(null); setDevices([]); setSelected(''); setHistory([]);
      }
      if (!accepting.current || event.attempt !== generation.current) { return; }
      setPhase(event.phase);
      if (event.kind === 'device') {
        setDevices(old => old.includes(event.device) ? old : [...old, event.device].slice(0, 8));
      } else {
        setHistory(old => [...old, event.phase].slice(-8));
      }
      if (event.kind === 'verified' && AppState.currentState === 'active') {
        setResult(event);
      }
      if (event.kind === 'failure' || event.kind === 'closed' ||
          event.phase.startsWith('No Pi') || event.phase.startsWith('Select')) {
        setBusy(false);
      }
    });
    // Native lifecycle also tears down independently of this JS observer.
    const lifecycle = AppState.addEventListener('change', state => {
      if (state !== 'active') { accepting.current = false; setBusy(false); setPhase('Foreground ended · scan again'); }
    });
    return () => { events.remove(); lifecycle.remove(); QTooth.disconnect(); };
  }, []);

  /** @brief Request only Android's required foreground Bluetooth permissions before scanning. */
  async function scan(): Promise<void> {
    accepting.current = false;
    setResult(null); setDevices([]); setSelected(''); setHistory([]); setBusy(true);
    try {
      const permissions = Number(Platform.Version) >= 31
        ? [PermissionsAndroid.PERMISSIONS.BLUETOOTH_SCAN, PermissionsAndroid.PERMISSIONS.BLUETOOTH_CONNECT]
        : [PermissionsAndroid.PERMISSIONS.ACCESS_FINE_LOCATION];
      const granted = await PermissionsAndroid.requestMultiple(permissions);
      if (!permissions.every(permission => granted[permission] === PermissionsAndroid.RESULTS.GRANTED)) {
        setPhase('Allow Nearby devices in Android Settings, then scan again'); setBusy(false); return;
      }
      QTooth.scan();
    } catch {
      setPhase('Permission request failed · check Android Settings'); setBusy(false);
    }
  }

  return (
    <SafeAreaView style={styles.page}>
      <StatusBar barStyle="light-content" />
      <View style={styles.header}>
        <Text style={styles.brand}>QUANTUMTOOTH</Text><Text style={styles.badge}>DEBUG</Text>
      </View>
      <Text style={styles.subtitle}>Post-quantum messaging over BLE</Text>
      <Text style={styles.route}>RASPBERRY PI 4  →  PIXEL 7a</Text>
      <View style={styles.status}>
        <Text style={styles.eyebrow}>CURRENT LINK / SESSION</Text>
        <Text accessibilityLiveRegion="polite" style={styles.phase}>{phase}</Text>
        <Text style={styles.algorithms}>ML-KEM-768  ·  AES-256-GCM</Text>
      </View>
      <View style={styles.cipher}>
        <Text style={styles.eyebrow}>ENCRYPTED RECORD · CIPHERTEXT EXCERPT</Text>
        <Text numberOfLines={2} style={styles.hex}>{result?.ciphertext || 'Waiting for an authenticated record'}</Text>
      </View>
      <View style={[styles.result, result !== null && styles.verified]}>
        <Text style={[styles.eyebrow, result !== null && styles.green]}>
          {result ? '✓ LAST VERIFIED RESULT ON PHONE' : 'VERIFIED RESULT ON PHONE'}
        </Text>
        <Text accessibilityLiveRegion="polite" numberOfLines={4} adjustsFontSizeToFit minimumFontScale={0.65}
          style={[styles.plaintext, result !== null && styles.green]}>{result?.plaintext || 'Awaiting verified text'}</Text>
        <Text style={styles.caption}>{result ? 'Authentication tag verified · retained after closure' : 'Released only after native authentication'}</Text>
      </View>
      <View style={styles.metrics}>
        <Text style={styles.metric}>MTU {result?.mtu ?? '—'}  ·  RX fragments {result?.fragments ?? '—'}</Text>
        <Text style={styles.metric}>Sequence {result?.sequence ?? '—'}  ·  Record {result?.recordBytes ?? '—'} bytes</Text>
        <Text style={styles.metric}>Handshake {result ? `${Math.round(result.handshakeMs)} ms` : '—'}</Text>
      </View>
      <View style={styles.spacer} />
      <View style={styles.controls}>
        <Pressable accessibilityRole="button" disabled={busy} onPress={() => { void scan(); }}
          style={[styles.button, busy && styles.disabled]}><Text style={styles.buttonText}>Scan</Text></Pressable>
        <Pressable accessibilityRole="button" onPress={() => { QTooth.disconnect(); }} style={styles.secondary}>
          <Text style={styles.secondaryText}>Disconnect</Text>
        </Pressable>
      </View>
      {devices.length > 0 && <View style={styles.selection}>
        <View style={styles.peers}>{devices.map(device => <Pressable key={device} accessibilityRole="button"
          accessibilityState={{selected: selected === device}} onPress={() => setSelected(device)}
          style={[styles.peer, selected === device && styles.selected]}><Text style={styles.secondaryText}>{device}</Text></Pressable>)}</View>
        <Pressable accessibilityRole="button" disabled={!selected} style={[styles.button, !selected && styles.disabled]}
          onPress={() => { setBusy(true); setDevices([]); QTooth.connect(selected); }}><Text style={styles.buttonText}>Connect selected Pi</Text></Pressable>
      </View>}
      <Pressable accessibilityRole="button" onPress={() => setDetails(true)} style={styles.detailsButton}>
        <Text style={styles.secondaryText}>Details · scoped diagnostics</Text>
      </Pressable>
      <Modal visible={details} animationType="none" onRequestClose={() => setDetails(false)}>
        <SafeAreaView style={styles.page}>
          <Text style={styles.brand}>DIAGNOSTICS</Text>
          <ScrollView style={styles.details}>
            <Text style={styles.caption}>MTU is observed by Android (23 until negotiation). Counts, sequence and length describe the displayed verified record. Ciphertext excerpt: at most 24 bytes. Handshake runs from ClientHello submission to verified PiFinish, excluding scan/setup. Unmeasured values show —.</Text>
            {result && <Text selectable style={styles.fullText}>{result.plaintext}</Text>}
            <Text style={styles.eyebrow}>LAST EIGHT PHASE EVENTS · CURRENT ATTEMPT</Text>
            {history.map((line, index) => <Text key={index} style={styles.metric}>{line}</Text>)}
          </ScrollView>
          <Pressable accessibilityRole="button" onPress={() => setDetails(false)} style={styles.button}><Text style={styles.buttonText}>Back</Text></Pressable>
        </SafeAreaView>
      </Modal>
    </SafeAreaView>
  );
}

/** @brief System-inset provider keeps the primary proof and controls on the portrait screen. */
export default function App(): React.JSX.Element {
  return <SafeAreaProvider><Screen /></SafeAreaProvider>;
}

/** @brief Compact Pixel portrait layout; only optional details scroll. */
const styles = StyleSheet.create({
  page: {flex: 1, backgroundColor: '#080E13', paddingHorizontal: 20, paddingVertical: 12},
  header: {flexDirection: 'row', alignItems: 'center', justifyContent: 'space-between'},
  brand: {color: '#F0F7F9', fontSize: 22, fontWeight: '800', letterSpacing: 1},
  badge: {color: '#4DE3EE', fontSize: 11, fontFamily: 'monospace'},
  subtitle: {color: '#A8BAC6', fontSize: 13, marginTop: 4},
  route: {color: '#4DE3EE', fontFamily: 'monospace', fontSize: 12, marginTop: 16},
  status: {marginTop: 14, borderLeftWidth: 2, borderColor: '#4DE3EE', paddingLeft: 12},
  eyebrow: {color: '#A8BAC6', fontSize: 10, fontWeight: '700', letterSpacing: 0.7},
  phase: {color: '#4DE3EE', fontSize: 16, fontWeight: '600', marginVertical: 5},
  algorithms: {color: '#A8BAC6', fontSize: 11, fontFamily: 'monospace'},
  cipher: {padding: 12, marginTop: 16, borderRadius: 8, backgroundColor: '#111E28'},
  hex: {color: '#92C5D1', fontFamily: 'monospace', fontSize: 12, marginTop: 8},
  result: {minHeight: 154, padding: 16, marginTop: 12, borderWidth: 1, borderColor: '#263B48', borderRadius: 10, justifyContent: 'center'},
  verified: {borderColor: '#62F5A4', backgroundColor: '#0D241B'},
  green: {color: '#62F5A4'},
  plaintext: {color: '#A8BAC6', fontSize: 32, fontWeight: '700', marginVertical: 12},
  caption: {color: '#A8BAC6', fontSize: 12, lineHeight: 19},
  metrics: {marginTop: 12, gap: 4},
  metric: {color: '#A8BAC6', fontFamily: 'monospace', fontSize: 12, lineHeight: 20},
  spacer: {flexGrow: 1, minHeight: 8},
  controls: {flexDirection: 'row', gap: 12},
  button: {backgroundColor: '#4DE3EE', paddingVertical: 12, paddingHorizontal: 20, borderRadius: 7, alignItems: 'center', flexGrow: 1},
  buttonText: {color: '#08141C', fontWeight: '700', fontSize: 14},
  secondary: {padding: 12, borderWidth: 1, borderColor: '#385464', borderRadius: 7, flexGrow: 1, alignItems: 'center'},
  secondaryText: {color: '#BDD3DF', fontSize: 13},
  disabled: {opacity: 0.4},
  selection: {marginTop: 8, gap: 8},
  peers: {flexDirection: 'row', flexWrap: 'wrap', gap: 4},
  peer: {padding: 6, borderWidth: 1, borderColor: '#385464', borderRadius: 4},
  selected: {borderColor: '#4DE3EE', backgroundColor: '#17323D'},
  detailsButton: {alignItems: 'center', paddingVertical: 12},
  details: {flex: 1, marginVertical: 20},
  fullText: {color: '#62F5A4', fontSize: 20, marginVertical: 20},
});
