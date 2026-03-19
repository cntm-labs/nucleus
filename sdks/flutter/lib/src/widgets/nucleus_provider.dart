import 'package:flutter/widgets.dart';
import '../auth/auth_state.dart';

class NucleusProvider extends InheritedNotifier<NucleusAuth> {
  const NucleusProvider({super.key, required NucleusAuth auth, required super.child})
    : super(notifier: auth);

  static NucleusAuth of(BuildContext context) {
    final provider = context.dependOnInheritedWidgetOfExactType<NucleusProvider>();
    if (provider == null) throw FlutterError('NucleusProvider not found');
    return provider.notifier!;
  }
}
