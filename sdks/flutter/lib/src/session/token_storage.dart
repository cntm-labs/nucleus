import 'package:flutter_secure_storage/flutter_secure_storage.dart';

class TokenStorage {
  static const _storage = FlutterSecureStorage();
  static const _sessionKey = 'nucleus_session_token';
  static const _refreshKey = 'nucleus_refresh_token';
  static const _expiresKey = 'nucleus_expires_at';
  static const _biometricKey = 'nucleus_biometric_enabled';

  static Future<void> saveSession(String token) => _storage.write(key: _sessionKey, value: token);
  static Future<String?> getSession() => _storage.read(key: _sessionKey);
  static Future<void> saveRefresh(String token) => _storage.write(key: _refreshKey, value: token);
  static Future<String?> getRefresh() => _storage.read(key: _refreshKey);
  static Future<void> saveExpiresAt(String expiresAt) => _storage.write(key: _expiresKey, value: expiresAt);
  static Future<String?> getExpiresAt() => _storage.read(key: _expiresKey);

  static Future<void> saveBiometricEnabled(bool enabled) =>
      _storage.write(key: _biometricKey, value: enabled.toString());

  static Future<bool> getBiometricEnabled() async {
    final val = await _storage.read(key: _biometricKey);
    return val == 'true';
  }

  static Future<void> clear() async {
    await _storage.delete(key: _sessionKey);
    await _storage.delete(key: _refreshKey);
    await _storage.delete(key: _expiresKey);
    // Biometric preference is intentionally preserved across sign-outs so the
    // user does not need to re-enable it on each new session.
  }
}
