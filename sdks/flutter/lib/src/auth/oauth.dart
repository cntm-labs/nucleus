import 'package:url_launcher/url_launcher.dart';
import '../client.dart';
import '../models/user.dart';
import '../models/session.dart';

class NucleusOAuth {
  final NucleusApiClient _api;

  NucleusOAuth(this._api);

  Future<void> launchOAuth(String provider, {String redirectUri = 'nucleus://oauth/callback'}) async {
    final url = _api.getOAuthUrl(provider, redirectUri);
    final uri = Uri.parse(url);
    if (await canLaunchUrl(uri)) {
      await launchUrl(uri, mode: LaunchMode.externalApplication);
    } else {
      throw Exception('Could not launch OAuth URL for $provider');
    }
  }

  Future<({NucleusUser user, NucleusSession session})> handleCallback(String code, {String redirectUri = 'nucleus://oauth/callback'}) {
    return _api.exchangeOAuthCode(code, redirectUri);
  }
}
