class NucleusActiveSession {
  final String id;
  final String userId;
  final String? deviceType;
  final String? deviceName;
  final String? browser;
  final String? ip;
  final String? countryCode;
  final DateTime createdAt;
  final DateTime lastActiveAt;

  NucleusActiveSession({
    required this.id,
    required this.userId,
    this.deviceType,
    this.deviceName,
    this.browser,
    this.ip,
    this.countryCode,
    required this.createdAt,
    required this.lastActiveAt,
  });

  factory NucleusActiveSession.fromJson(Map<String, dynamic> json) =>
      NucleusActiveSession(
        id: json['id'] as String? ?? '',
        userId: json['user_id'] as String? ?? '',
        deviceType: json['device_type'] as String?,
        deviceName: json['device_name'] as String?,
        browser: json['browser'] as String?,
        ip: json['ip'] as String?,
        countryCode: json['country_code'] as String?,
        createdAt: DateTime.parse(json['created_at'] as String),
        lastActiveAt: DateTime.parse(json['last_active_at'] as String),
      );
}
