import 'config.dart';
import 'client.dart';
import 'auth/auth_state.dart';
import 'session/token_storage.dart';
import 'session/auto_refresh.dart';
import 'models/session.dart';

class Nucleus {
  static NucleusApiClient? _client;
  static NucleusAuth? _auth;
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

  static Future<void> configure(NucleusConfig config) async {
    _client = NucleusApiClient(config);
    _auth = NucleusAuth(_client!);
    _autoRefresh = AutoRefresh(onRefresh: _refreshSession);
    _configured = true;

    // Restore session from secure storage
    final token = await TokenStorage.getSession();
    final refreshToken = await TokenStorage.getRefresh();
    if (token != null && refreshToken != null) {
      _client!.setToken(token);
      try {
        final user = await _client!.getUser();
        _auth!.setUser(user);
        _auth!.setSession(NucleusSession(
          id: '', token: token, refreshToken: refreshToken,
          expiresAt: DateTime.now().add(const Duration(minutes: 5)),
        ));
        _autoRefresh!.start();
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
      _auth!.setSession(newSession);
    } catch (_) {
      await signOut();
    }
  }

  static Future<void> signOut() async {
    try { await _client?.signOut(); } catch (_) { /* best effort */ }
    _client?.setToken(null);
    _autoRefresh?.stop();
    await TokenStorage.clear();
    _auth?.signOut();
  }

  static void dispose() {
    _autoRefresh?.stop();
  }
}
