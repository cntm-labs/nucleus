import 'package:flutter/foundation.dart';
import '../client.dart';
import '../models/user.dart';
import '../models/session.dart';
import '../models/organization.dart';
import '../models/member.dart';
import '../session/token_storage.dart';

class NucleusAuth extends ChangeNotifier {
  final NucleusApiClient _api;

  NucleusUser? _user;
  NucleusSession? _session;
  NucleusOrganization? _organization;

  NucleusAuth(this._api);

  NucleusUser? get user => _user;
  NucleusSession? get session => _session;
  NucleusOrganization? get organization => _organization;
  bool get isSignedIn => _user != null && _session != null;
  String? get token => _session?.token;

  void setUser(NucleusUser? user) { _user = user; notifyListeners(); }
  void setSession(NucleusSession? session) { _session = session; notifyListeners(); }
  void setOrganization(NucleusOrganization? org) { _organization = org; notifyListeners(); }

  // --- Auth ---
  Future<({NucleusUser user, NucleusSession session})> signIn(String email, String password) async {
    final result = await _api.signIn(email, password);
    await _setAuthResult(result.user, result.session);
    return result;
  }

  Future<({NucleusUser user, NucleusSession session})> signUp(String email, String password, {String? firstName, String? lastName}) async {
    final result = await _api.signUp(email, password, firstName: firstName, lastName: lastName);
    await _setAuthResult(result.user, result.session);
    return result;
  }

  Future<void> signOut() async {
    _user = null; _session = null; _organization = null;
    _api.setToken(null);
    await TokenStorage.clear();
    notifyListeners();
  }

  // --- User Profile ---
  Future<NucleusUser> updateProfile({String? firstName, String? lastName, String? avatarUrl}) async {
    final user = await _api.updateUser(firstName: firstName, lastName: lastName, avatarUrl: avatarUrl);
    _user = user; notifyListeners();
    return user;
  }

  Future<void> updatePassword(String currentPassword, String newPassword) =>
    _api.updatePassword(currentPassword, newPassword);

  Future<void> updateEmail(String email) => _api.updateEmail(email);

  // --- MFA ---
  Future<({String secret, String qrUri})> mfaSetupTotp() => _api.mfaTotpSetup();
  Future<bool> mfaVerifyTotp(String code) => _api.mfaTotpVerify(code);
  Future<void> mfaSendSms(String phone) => _api.mfaSmsSend(phone);
  Future<bool> mfaVerifySms(String code) => _api.mfaSmsVerify(code);
  Future<List<String>> mfaGetBackupCodes() => _api.mfaBackupCodes();

  // --- Verification ---
  Future<void> sendEmailVerification(String email) => _api.sendEmailVerification(email);
  Future<bool> confirmEmailVerification(String code) => _api.confirmEmailVerification(code);
  Future<void> sendPhoneVerification(String phone) => _api.sendPhoneVerification(phone);
  Future<bool> confirmPhoneVerification(String code) => _api.confirmPhoneVerification(code);

  // --- Sessions ---
  Future<List<NucleusSession>> getSessions() => _api.getSessions();
  Future<void> revokeSession(String sessionId) => _api.revokeSession(sessionId);

  Future<NucleusSession> switchSession(String sessionId) async {
    final session = await _api.switchSession(sessionId);
    await _setAuthResult(await _api.getUser(), session);
    return session;
  }

  // --- Organizations ---
  Future<List<NucleusOrganization>> getOrganizations() => _api.getOrganizations();

  Future<NucleusOrganization> createOrganization(String name, String slug) =>
    _api.createOrganization(name, slug);

  Future<List<NucleusMember>> getMembers(String orgId) => _api.getMembers(orgId);
  Future<void> removeMember(String orgId, String memberId) => _api.removeMember(orgId, memberId);
  Future<NucleusMember> updateMemberRole(String orgId, String memberId, String role) => _api.updateMemberRole(orgId, memberId, role);
  Future<NucleusInvitation> inviteMember(String orgId, String email, String role) => _api.createInvitation(orgId, email, role);

  // --- Internal ---
  Future<void> _setAuthResult(NucleusUser user, NucleusSession session) async {
    _user = user;
    _session = session;
    _api.setToken(session.token);
    await TokenStorage.saveSession(session.token);
    await TokenStorage.saveRefresh(session.refreshToken);
    notifyListeners();
  }
}
