library nucleus_flutter;

import 'package:flutter/foundation.dart';

const nucleusVersion = '0.1.0-dev.1';

void _printDevWarning() {
  if (nucleusVersion.contains('dev')) {
    debugPrint('[Nucleus] WARNING: You are using a dev preview ($nucleusVersion). Do not use in production.');
  }
}

final _devWarningPrinted = () { _printDevWarning(); return true; }();
export 'src/config.dart';
export 'src/models/user.dart';
export 'src/models/session.dart';
export 'src/models/organization.dart';
export 'src/auth/auth_state.dart';
export 'src/widgets/nucleus_provider.dart';
export 'src/widgets/sign_in_widget.dart';
export 'src/widgets/sign_up_widget.dart';
export 'src/widgets/user_button.dart';
export 'src/widgets/org_switcher.dart';
