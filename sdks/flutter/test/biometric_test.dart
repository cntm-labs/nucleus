import 'package:flutter_test/flutter_test.dart';
import 'package:cntm_nucleus/nucleus.dart';

void main() {
  group('NucleusConfig biometricAuth flag', () {
    test('defaults to false', () {
      final config = NucleusConfig(publishableKey: 'pk_test');
      expect(config.biometricAuth, false);
    });

    test('can be enabled at construction', () {
      final config = NucleusConfig(publishableKey: 'pk_test', biometricAuth: true);
      expect(config.biometricAuth, true);
    });
  });

  group('BiometricAuth API surface', () {
    test('isAvailable is callable', () {
      expect(BiometricAuth.isAvailable, isA<Function>());
    });

    test('getAvailableBiometrics is callable', () {
      expect(BiometricAuth.getAvailableBiometrics, isA<Function>());
    });

    test('authenticate is callable', () {
      expect(BiometricAuth.authenticate, isA<Function>());
    });
  });

  group('Nucleus biometric methods API surface', () {
    test('isBiometricAvailable is callable', () {
      expect(Nucleus.isBiometricAvailable, isA<Function>());
    });

    test('isBiometricLockEnabled is callable', () {
      expect(Nucleus.isBiometricLockEnabled, isA<Function>());
    });

    test('enableBiometricLock is callable', () {
      expect(Nucleus.enableBiometricLock, isA<Function>());
    });

    test('disableBiometricLock is callable', () {
      expect(Nucleus.disableBiometricLock, isA<Function>());
    });

    test('unlockWithBiometrics is callable', () {
      expect(Nucleus.unlockWithBiometrics, isA<Function>());
    });

    test('unlockWithBiometrics throws when not configured', () {
      expect(
        () => Nucleus.unlockWithBiometrics(),
        throwsA(isA<StateError>()),
      );
    });
  });
}
