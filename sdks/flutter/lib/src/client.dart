import 'dart:convert';
import 'package:http/http.dart' as http;
import 'config.dart';
import 'models/user.dart';
import 'models/session.dart';
import 'models/organization.dart';
import 'models/member.dart';

class NucleusApiClient {
  final NucleusConfig config;
  String? _accessToken;

  NucleusApiClient(this.config);
  void setToken(String? token) { _accessToken = token; }

  Future<Map<String, dynamic>> _get(String path) async {
    final res = await http.get(Uri.parse('${config.effectiveBaseUrl}$path'), headers: _headers());
    return _handleResponse(res);
  }

  Future<Map<String, dynamic>> _post(String path, {Map<String, dynamic>? body}) async {
    final res = await http.post(Uri.parse('${config.effectiveBaseUrl}$path'),
      headers: _headers(), body: body != null ? jsonEncode(body) : null);
    return _handleResponse(res);
  }

  Future<Map<String, dynamic>> _put(String path, {Map<String, dynamic>? body}) async {
    final res = await http.put(Uri.parse('${config.effectiveBaseUrl}$path'),
      headers: _headers(), body: body != null ? jsonEncode(body) : null);
    return _handleResponse(res);
  }

  Future<Map<String, dynamic>> _patch(String path, {Map<String, dynamic>? body}) async {
    final res = await http.patch(Uri.parse('${config.effectiveBaseUrl}$path'),
      headers: _headers(), body: body != null ? jsonEncode(body) : null);
    return _handleResponse(res);
  }

  Future<void> _delete(String path) async {
    final res = await http.delete(Uri.parse('${config.effectiveBaseUrl}$path'), headers: _headers());
    if (res.statusCode >= 300) throw NucleusApiException(res.statusCode, res.body);
  }

  Map<String, String> _headers() => {
    'Content-Type': 'application/json',
    if (_accessToken != null) 'Authorization': 'Bearer $_accessToken',
    'X-Nucleus-Key': config.publishableKey,
  };

  Map<String, dynamic> _handleResponse(http.Response res) {
    if (res.statusCode >= 200 && res.statusCode < 300) {
      if (res.body.isEmpty) return {};
      return jsonDecode(res.body);
    }
    throw NucleusApiException(res.statusCode, res.body);
  }

  // --- Auth ---
  Future<({NucleusUser user, NucleusSession session})> signIn(String email, String password) async {
    final json = await _post('/v1/auth/sign-in', body: {'email': email, 'password': password});
    return (user: NucleusUser.fromJson(json['user']), session: NucleusSession.fromJson(json['session']));
  }

  Future<({NucleusUser user, NucleusSession session})> signUp(String email, String password, {String? firstName, String? lastName}) async {
    final json = await _post('/v1/auth/sign-up', body: {
      'email': email, 'password': password,
      if (firstName != null) 'first_name': firstName,
      if (lastName != null) 'last_name': lastName,
    });
    return (user: NucleusUser.fromJson(json['user']), session: NucleusSession.fromJson(json['session']));
  }

  Future<void> signOut() => _post('/v1/auth/sign-out').then((_) {});

  // --- OAuth ---
  String getOAuthUrl(String provider, String redirectUri, {String? state}) {
    final params = <String, String>{
      'redirect_uri': redirectUri,
      'publishable_key': config.publishableKey,
    };
    if (state != null) params['state'] = state;
    final query = params.entries.map((e) => '${Uri.encodeComponent(e.key)}=${Uri.encodeComponent(e.value)}').join('&');
    return '${config.effectiveBaseUrl}/v1/oauth/$provider/authorize?$query';
  }

  Future<({NucleusUser user, NucleusSession session})> exchangeOAuthCode(String code, String redirectUri) async {
    final json = await _post('/v1/oauth/token', body: {'code': code, 'redirect_uri': redirectUri});
    return (user: NucleusUser.fromJson(json['user']), session: NucleusSession.fromJson(json['session']));
  }

  // --- MFA ---
  Future<({String secret, String qrUri})> mfaTotpSetup() async {
    final json = await _post('/v1/auth/mfa/totp/setup');
    return (secret: json['secret'] as String, qrUri: json['qr_uri'] as String);
  }

  Future<bool> mfaTotpVerify(String code) async {
    final json = await _post('/v1/auth/mfa/totp/verify', body: {'code': code});
    return json['verified'] as bool;
  }

  Future<void> mfaSmsSend(String phone) => _post('/v1/auth/mfa/sms/send', body: {'phone': phone}).then((_) {});

  Future<bool> mfaSmsVerify(String code) async {
    final json = await _post('/v1/auth/mfa/sms/verify', body: {'code': code});
    return json['verified'] as bool;
  }

  Future<List<String>> mfaBackupCodes() async {
    final json = await _get('/v1/auth/mfa/backup-codes');
    return List<String>.from(json['codes']);
  }

  // --- Verification ---
  Future<void> sendEmailVerification(String email) => _post('/v1/auth/verify-email/send', body: {'email': email}).then((_) {});
  Future<bool> confirmEmailVerification(String code) async {
    final json = await _post('/v1/auth/verify-email/confirm', body: {'code': code});
    return json['verified'] as bool;
  }
  Future<void> sendPhoneVerification(String phone) => _post('/v1/auth/verify-phone/send', body: {'phone': phone}).then((_) {});
  Future<bool> confirmPhoneVerification(String code) async {
    final json = await _post('/v1/auth/verify-phone/confirm', body: {'code': code});
    return json['verified'] as bool;
  }

  // --- Sessions ---
  Future<List<NucleusSession>> getSessions() async {
    final res = await http.get(Uri.parse('${config.effectiveBaseUrl}/v1/sessions'), headers: _headers());
    if (res.statusCode >= 300) throw NucleusApiException(res.statusCode, res.body);
    final decoded = jsonDecode(res.body);
    if (decoded is List) return decoded.map((e) => NucleusSession.fromJson(e)).toList();
    if (decoded is Map && decoded.containsKey('sessions')) return (decoded['sessions'] as List).map((e) => NucleusSession.fromJson(e)).toList();
    return [];
  }

  Future<NucleusSession> refreshSession(String refreshToken) async {
    final json = await _post('/v1/sessions/refresh', body: {'refresh_token': refreshToken});
    return NucleusSession.fromJson(json);
  }

  Future<void> revokeSession(String sessionId) => _delete('/v1/sessions/$sessionId');

  Future<NucleusSession> switchSession(String sessionId) async {
    final json = await _post('/v1/sessions/switch', body: {'session_id': sessionId});
    return NucleusSession.fromJson(json);
  }

  // --- User Profile ---
  Future<NucleusUser> getUser() async {
    final json = await _get('/v1/user');
    return NucleusUser.fromJson(json);
  }

  Future<NucleusUser> updateUser({String? firstName, String? lastName, String? avatarUrl}) async {
    final json = await _patch('/v1/user', body: {
      if (firstName != null) 'first_name': firstName,
      if (lastName != null) 'last_name': lastName,
      if (avatarUrl != null) 'avatar_url': avatarUrl,
    });
    return NucleusUser.fromJson(json);
  }

  Future<void> updatePassword(String currentPassword, String newPassword) =>
    _put('/v1/user/password', body: {'current_password': currentPassword, 'new_password': newPassword}).then((_) {});

  Future<void> updateEmail(String email) => _put('/v1/user/email', body: {'email': email}).then((_) {});

  // --- Organizations ---
  Future<List<NucleusOrganization>> getOrganizations() async {
    final res = await http.get(Uri.parse('${config.effectiveBaseUrl}/v1/organizations'), headers: _headers());
    if (res.statusCode >= 300) throw NucleusApiException(res.statusCode, res.body);
    final decoded = jsonDecode(res.body);
    if (decoded is List) return decoded.map((e) => NucleusOrganization.fromJson(e)).toList();
    if (decoded is Map && decoded.containsKey('organizations')) return (decoded['organizations'] as List).map((e) => NucleusOrganization.fromJson(e)).toList();
    return [];
  }

  Future<NucleusOrganization> createOrganization(String name, String slug) async {
    final json = await _post('/v1/organizations', body: {'name': name, 'slug': slug});
    return NucleusOrganization.fromJson(json);
  }

  Future<List<NucleusMember>> getMembers(String orgId) async {
    final res = await http.get(Uri.parse('${config.effectiveBaseUrl}/v1/organizations/$orgId/members'), headers: _headers());
    if (res.statusCode >= 300) throw NucleusApiException(res.statusCode, res.body);
    final decoded = jsonDecode(res.body);
    if (decoded is List) return decoded.map((e) => NucleusMember.fromJson(e)).toList();
    if (decoded is Map && decoded.containsKey('members')) return (decoded['members'] as List).map((e) => NucleusMember.fromJson(e)).toList();
    return [];
  }

  Future<void> removeMember(String orgId, String memberId) => _delete('/v1/organizations/$orgId/members/$memberId');

  Future<NucleusMember> updateMemberRole(String orgId, String memberId, String role) async {
    final json = await _put('/v1/organizations/$orgId/members/$memberId/role', body: {'role': role});
    return NucleusMember.fromJson(json);
  }

  Future<NucleusInvitation> createInvitation(String orgId, String email, String role) async {
    final json = await _post('/v1/organizations/$orgId/invitations', body: {'email': email, 'role': role});
    return NucleusInvitation.fromJson(json);
  }

  Future<NucleusInvitation> updateInvitation(String orgId, String invitationId, String action) async {
    final json = await _put('/v1/organizations/$orgId/invitations/$invitationId', body: {'action': action});
    return NucleusInvitation.fromJson(json);
  }
}

class NucleusApiException implements Exception {
  final int statusCode;
  final String body;
  NucleusApiException(this.statusCode, this.body);
  @override String toString() => 'NucleusApiException($statusCode): $body';
}
