class NucleusMember {
  final String id, userId, orgId, role, email;
  final List<String> permissions;
  final String? firstName, lastName;

  NucleusMember({
    required this.id, required this.userId, required this.orgId,
    required this.role, required this.email, this.permissions = const [],
    this.firstName, this.lastName,
  });

  factory NucleusMember.fromJson(Map<String, dynamic> json) => NucleusMember(
    id: json['id'], userId: json['user_id'], orgId: json['org_id'],
    role: json['role'], email: json['email'],
    permissions: List<String>.from(json['permissions'] ?? []),
    firstName: json['first_name'], lastName: json['last_name'],
  );
}

class NucleusInvitation {
  final String id, orgId, email, role, status;
  final DateTime createdAt;

  NucleusInvitation({
    required this.id, required this.orgId, required this.email,
    required this.role, required this.status, required this.createdAt,
  });

  factory NucleusInvitation.fromJson(Map<String, dynamic> json) => NucleusInvitation(
    id: json['id'], orgId: json['org_id'], email: json['email'],
    role: json['role'], status: json['status'],
    createdAt: DateTime.parse(json['created_at']),
  );
}
