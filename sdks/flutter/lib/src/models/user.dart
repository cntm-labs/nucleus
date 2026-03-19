class NucleusUser {
  final String id, email;
  final bool emailVerified;
  final String? firstName, lastName, avatarUrl, username;
  final Map<String, dynamic> metadata;
  final DateTime createdAt;

  NucleusUser({required this.id, required this.email, this.emailVerified = false,
    this.firstName, this.lastName, this.avatarUrl, this.username,
    this.metadata = const {}, required this.createdAt});

  factory NucleusUser.fromJson(Map<String, dynamic> json) => NucleusUser(
    id: json['id'], email: json['email'], emailVerified: json['email_verified'] ?? false,
    firstName: json['first_name'], lastName: json['last_name'],
    avatarUrl: json['avatar_url'], username: json['username'],
    metadata: json['metadata'] ?? {}, createdAt: DateTime.parse(json['created_at']));

  String get fullName => [firstName, lastName].whereType<String>().join(' ');
}
