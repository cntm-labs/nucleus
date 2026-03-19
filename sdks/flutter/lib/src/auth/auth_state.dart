import 'package:flutter/foundation.dart';
import '../models/user.dart';
import '../models/session.dart';
import '../models/organization.dart';

class NucleusAuth extends ChangeNotifier {
  NucleusUser? _user;
  NucleusSession? _session;
  NucleusOrganization? _organization;

  NucleusUser? get user => _user;
  NucleusSession? get session => _session;
  NucleusOrganization? get organization => _organization;
  bool get isSignedIn => _user != null && _session != null;

  void setUser(NucleusUser? user) { _user = user; notifyListeners(); }
  void setSession(NucleusSession? session) { _session = session; notifyListeners(); }
  void setOrganization(NucleusOrganization? org) { _organization = org; notifyListeners(); }

  Future<void> signOut() async {
    _user = null; _session = null; _organization = null;
    notifyListeners();
  }

  Future<String?> getToken() async => _session?.token;
}
