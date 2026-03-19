import 'dart:async';

class AutoRefresh {
  Timer? _timer;
  final Future<void> Function() onRefresh;
  AutoRefresh({required this.onRefresh});

  void start({Duration interval = const Duration(minutes: 4)}) {
    _timer?.cancel();
    _timer = Timer.periodic(interval, (_) => onRefresh());
  }

  void stop() { _timer?.cancel(); _timer = null; }
}
