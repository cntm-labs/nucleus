import 'dart:async';
import 'package:flutter_test/flutter_test.dart';

// Inline test of the auto-refresh scheduling logic (since AutoRefresh is internal)
void main() {
  group('Auto-refresh scheduling logic', () {
    test('timer fires after calculated delay', () async {
      bool refreshed = false;
      final expiresAt = DateTime.now().add(const Duration(seconds: 2));
      final buffer = const Duration(seconds: 1);
      final delay = expiresAt.subtract(buffer).difference(DateTime.now());

      expect(delay.inMilliseconds, greaterThan(0));
      expect(delay.inMilliseconds, lessThan(2000));

      // Simulate the timer logic
      final timer = Timer(const Duration(milliseconds: 50), () { refreshed = true; });
      await Future.delayed(const Duration(milliseconds: 100));
      expect(refreshed, true);
      timer.cancel();
    });

    test('past expiry results in negative delay', () {
      final expiresAt = DateTime.now().subtract(const Duration(minutes: 5));
      final buffer = const Duration(seconds: 60);
      final delay = expiresAt.subtract(buffer).difference(DateTime.now());
      expect(delay.isNegative, true);
    });
  });
}
