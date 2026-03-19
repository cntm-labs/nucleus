class NucleusConfig {
  final String publishableKey;
  final String? apiBaseUrl;
  final bool biometricAuth;
  NucleusConfig({ required this.publishableKey, this.apiBaseUrl, this.biometricAuth = false });
  String get effectiveBaseUrl => apiBaseUrl ?? 'https://api.nucleus.dev';
}
