/** @file @brief Codegen contract: JS receives safe events, never protocol bytes or keys. */
import type {TurboModule, CodegenTypes} from 'react-native';
import {TurboModuleRegistry} from 'react-native';

/** @brief Bounded sanitized event; sequence is decimal text to avoid JS precision loss. */
export type ProofEvent = {
  kind: string;
  phase: string;
  attempt: number;
  device: string;
  plaintext: string;
  ciphertext: string;
  sequence: string;
  mtu: number;
  fragments: number;
  recordBytes: number;
  handshakeMs: number;
};
/** @brief Foreground actions and native-owned event stream. */
export interface Spec extends TurboModule {
  scan(): void;
  connect(device: string): void;
  disconnect(): void;
  readonly onEvent: CodegenTypes.EventEmitter<ProofEvent>;
}
export default TurboModuleRegistry.getEnforcing<Spec>('QTooth');
