class NucleusSession {
  final String id, token, refreshToken, userId;
  final DateTime expiresAt;

  NucleusSession({
    required this.id, required this.token, required this.refreshToken,
    required this.expiresAt, this.userId = '',
  });

  bool get isExpired => DateTime.now().isAfter(expiresAt);

  factory NucleusSession.fromJson(Map<String, dynamic> json) => NucleusSession(
    id: json['id'] ?? '',
    token: json['token'] ?? '',
    refreshToken: json['refresh_token'] ?? '',
    expiresAt: DateTime.parse(json['expires_at']),
    userId: json['user_id'] ?? '',
  );
}
