import 'dart:convert';
import 'package:http/http.dart' as http;
import 'config.dart';

class NucleusApiClient {
  final NucleusConfig config;
  String? _accessToken;

  NucleusApiClient(this.config);
  void setToken(String? token) { _accessToken = token; }

  Future<Map<String, dynamic>> get(String path) async {
    final res = await http.get(Uri.parse('${config.effectiveBaseUrl}$path'),
      headers: _headers());
    return _handleResponse(res);
  }

  Future<Map<String, dynamic>> post(String path, {Map<String, dynamic>? body}) async {
    final res = await http.post(Uri.parse('${config.effectiveBaseUrl}$path'),
      headers: _headers(), body: body != null ? jsonEncode(body) : null);
    return _handleResponse(res);
  }

  Map<String, String> _headers() => {
    'Content-Type': 'application/json',
    if (_accessToken != null) 'Authorization': 'Bearer $_accessToken',
    'X-Nucleus-Key': config.publishableKey,
  };

  Map<String, dynamic> _handleResponse(http.Response res) {
    if (res.statusCode >= 200 && res.statusCode < 300) return jsonDecode(res.body);
    throw NucleusApiException(res.statusCode, res.body);
  }
}

class NucleusApiException implements Exception {
  final int statusCode;
  final String body;
  NucleusApiException(this.statusCode, this.body);
  @override String toString() => 'NucleusApiException($statusCode): $body';
}
