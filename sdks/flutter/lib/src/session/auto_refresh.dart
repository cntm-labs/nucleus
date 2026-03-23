import 'dart:async';

class AutoRefresh {
  Timer? _timer;
  final Future<void> Function() onRefresh;
  AutoRefresh({required this.onRefresh});

  void scheduleAt(DateTime expiresAt, {Duration buffer = const Duration(seconds: 60)}) {
    _timer?.cancel();
    final refreshAt = expiresAt.subtract(buffer).difference(DateTime.now());
    if (refreshAt.isNegative) {
      onRefresh();
      return;
    }
    _timer = Timer(refreshAt, () => onRefresh());
  }

  void stop() { _timer?.cancel(); _timer = null; }
}
