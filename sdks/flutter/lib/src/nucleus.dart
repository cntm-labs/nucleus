import 'config.dart';
import 'client.dart';
import 'auth/auth_state.dart';
import 'auth/oauth.dart';
import 'session/token_storage.dart';
import 'session/auto_refresh.dart';
import 'biometric/biometric_auth.dart';
import 'models/session.dart';

class Nucleus {
  static NucleusApiClient? _client;
  static NucleusAuth? _auth;
  static NucleusOAuth? _oauth;
  static AutoRefresh? _autoRefresh;
  static bool _configured = false;

  static NucleusApiClient get client {
    if (_client == null) throw StateError('Nucleus not configured. Call Nucleus.configure() first.');
    return _client!;
  }

  static NucleusAuth get auth {
    if (_auth == null) throw StateError('Nucleus not configured. Call Nucleus.configure() first.');
    return _auth!;
  }

  static bool get isConfigured => _configured;

  static NucleusOAuth get oauth {
    if (_client == null) throw StateError('Nucleus not configured. Call Nucleus.configure() first.');
    _oauth ??= NucleusOAuth(_client!);
    return _oauth!;
  }

  static Future<void> configure(NucleusConfig config) async {
    _client = NucleusApiClient(config);
    _auth = NucleusAuth(_client!);
    _autoRefresh = AutoRefresh(onRefresh: _refreshSession);
    _configured = true;

    // When biometric lock is active, the session stays locked in secure storage
    // until the user calls unlockWithBiometrics().
    final biometricLockActive =
        config.biometricAuth && await TokenStorage.getBiometricEnabled();
    if (!biometricLockActive) {
      await _restoreSession();
    }
  }

  static Future<void> _restoreSession() async {
    final token = await TokenStorage.getSession();
    final refreshToken = await TokenStorage.getRefresh();
    final expiresAtStr = await TokenStorage.getExpiresAt();
    if (token != null && refreshToken != null) {
      _client!.setToken(token);
      try {
        final user = await _client!.getUser();
        final expiresAt = expiresAtStr != null
            ? DateTime.parse(expiresAtStr)
            : DateTime.now().add(const Duration(minutes: 5));
        final session = NucleusSession(
          id: '', token: token, refreshToken: refreshToken, expiresAt: expiresAt,
        );
        _auth!.setUser(user);
        _auth!.setSession(session);
        _autoRefresh!.scheduleAt(expiresAt);
      } catch (_) {
        await TokenStorage.clear();
        _client!.setToken(null);
      }
    }
  }

  // --- Biometrics ---

  /// Returns `true` if the device supports biometric authentication and has
  /// enrolled biometrics (FaceID, fingerprint, etc.).
  static Future<bool> isBiometricAvailable() => BiometricAuth.isAvailable();

  /// Returns `true` if the user has enabled biometric unlock for their session.
  static Future<bool> isBiometricLockEnabled() => TokenStorage.getBiometricEnabled();

  /// Enables biometric unlock for this user's session.
  ///
  /// Requires `biometricAuth: true` in [NucleusConfig] and an active sign-in.
  /// The user is prompted once to confirm their biometric before the lock is
  /// activated. On subsequent cold starts, [configure] will leave the session
  /// locked until [unlockWithBiometrics] is called.
  ///
  /// Throws [StateError] if the feature is not enabled in config, the user is
  /// not signed in, biometrics are unavailable on the device, or authentication
  /// fails.
  static Future<void> enableBiometricLock({String? reason}) async {
    if (!(_client?.config.biometricAuth ?? false)) {
      throw StateError(
        'Biometric auth must be enabled in NucleusConfig (biometricAuth: true).',
      );
    }
    if (_auth?.isSignedIn != true) {
      throw StateError('Must be signed in to enable biometric lock.');
    }
    if (!await BiometricAuth.isAvailable()) {
      throw StateError('Biometric authentication is not available on this device.');
    }
    final authenticated = await BiometricAuth.authenticate(
      reason: reason ?? 'Confirm your identity to enable biometric unlock',
    );
    if (!authenticated) throw StateError('Biometric authentication failed or was cancelled.');
    await TokenStorage.saveBiometricEnabled(true);
  }

  /// Disables biometric unlock. The session will auto-restore on the next cold
  /// start without a biometric prompt.
  static Future<void> disableBiometricLock() =>
      TokenStorage.saveBiometricEnabled(false);

  /// Prompts the user for biometric authentication (FaceID / fingerprint) and,
  /// on success, restores the persisted session from secure storage.
  ///
  /// Returns `true` if authentication succeeded and a valid session was
  /// restored, `false` if the user cancelled or authentication failed.
  ///
  /// Typical call site: your app's `AppLifecycleListener` or a dedicated lock
  /// screen widget when `Nucleus.auth.isSignedIn` is `false` after configure.
  static Future<bool> unlockWithBiometrics({String? reason}) async {
    if (!_configured) {
      throw StateError('Nucleus not configured. Call Nucleus.configure() first.');
    }
    final authenticated = await BiometricAuth.authenticate(
      reason: reason ?? 'Authenticate to access your account',
    );
    if (!authenticated) return false;
    await _restoreSession();
    return _auth?.isSignedIn ?? false;
  }

  // --- Session ---

  static Future<void> _refreshSession() async {
    final session = _auth?.session;
    if (session == null) return;
    try {
      final newSession = await _client!.refreshSession(session.refreshToken);
      _client!.setToken(newSession.token);
      await TokenStorage.saveSession(newSession.token);
      await TokenStorage.saveRefresh(newSession.refreshToken);
      await TokenStorage.saveExpiresAt(newSession.expiresAt.toIso8601String());
      _auth!.setSession(newSession);
      _autoRefresh?.scheduleAt(newSession.expiresAt);
    } catch (_) {
      await signOut();
    }
  }

  static Future<void> signOut() async {
    try { await _client?.signOut(); } catch (_) { /* best effort */ }
    _client?.setToken(null);
    _autoRefresh?.stop();
    await TokenStorage.clear();
    _auth?.clearAuthState();
  }

  static void dispose() {
    _autoRefresh?.stop();
  }
}
