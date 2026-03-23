import 'dart:math';
import 'package:url_launcher/url_launcher.dart';
import '../client.dart';
import '../models/user.dart';
import '../models/session.dart';

class NucleusOAuth {
  final NucleusApiClient _api;
  String? _pendingState;

  NucleusOAuth(this._api);

  String _generateState() {
    final random = Random.secure();
    final bytes = List<int>.generate(32, (_) => random.nextInt(256));
    return bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
  }

  Future<void> launchOAuth(String provider, {String redirectUri = 'nucleus://oauth/callback'}) async {
    _pendingState = _generateState();
    final url = _api.getOAuthUrl(provider, redirectUri, state: _pendingState);
    final uri = Uri.parse(url);
    if (await canLaunchUrl(uri)) {
      await launchUrl(uri, mode: LaunchMode.externalApplication);
    } else {
      _pendingState = null;
      throw Exception('Could not launch OAuth URL for $provider');
    }
  }

  Future<({NucleusUser user, NucleusSession session})> handleCallback(
    String code, {
    String redirectUri = 'nucleus://oauth/callback',
    String? state,
  }) {
    if (_pendingState == null || state != _pendingState) {
      _pendingState = null;
      throw Exception('OAuth state mismatch — possible CSRF attack');
    }
    _pendingState = null;
    return _api.exchangeOAuthCode(code, redirectUri);
  }
}
