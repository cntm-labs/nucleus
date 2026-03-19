class NucleusSession {
  final String id, token;
  final DateTime expiresAt;
  NucleusSession({required this.id, required this.token, required this.expiresAt});
  bool get isExpired => DateTime.now().isAfter(expiresAt);
}
