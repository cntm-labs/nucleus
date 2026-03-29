import 'config.dart';
import 'client.dart';
import 'auth/auth_state.dart';
import 'auth/oauth.dart';
import 'session/token_storage.dart';
import 'session/auto_refresh.dart';
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

    // Restore session from secure storage
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
