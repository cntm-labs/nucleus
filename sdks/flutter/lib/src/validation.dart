class ValidationError implements Exception {
  final String field;
  final String message;
  ValidationError(this.field, this.message);
  @override
  String toString() => 'ValidationError($field): $message';
}

void validateEmail(String email) {
  if (email.trim().isEmpty) throw ValidationError('email', 'Email is required');
  if (!RegExp(r'^[^\s@]+@[^\s@]+\.[^\s@]+$').hasMatch(email)) {
    throw ValidationError('email', 'Invalid email format');
  }
}

void validatePassword(String password) {
  if (password.isEmpty) throw ValidationError('password', 'Password is required');
  if (password.length < 8) {
    throw ValidationError('password', 'Password must be at least 8 characters');
  }
}
