library nucleus_flutter;

// Core
export 'src/config.dart';
export 'src/nucleus.dart';
export 'src/client.dart' show NucleusApiException;

// Models
export 'src/models/user.dart';
export 'src/models/session.dart';
export 'src/models/organization.dart';
export 'src/models/member.dart';

// Auth
export 'src/auth/auth_state.dart';
export 'src/auth/oauth.dart';

// Session
export 'src/session/token_storage.dart';

// Theme
export 'src/theme/nucleus_theme.dart';

// Widgets
export 'src/widgets/nucleus_provider.dart';
export 'src/widgets/sign_in_widget.dart';
export 'src/widgets/sign_up_widget.dart';
export 'src/widgets/user_button.dart';
export 'src/widgets/user_profile.dart';
export 'src/widgets/org_switcher.dart';
export 'src/widgets/org_profile.dart';

import 'package:flutter/foundation.dart';

const nucleusVersion = '0.1.0-dev.1';

void _printDevWarning() {
  if (nucleusVersion.contains('dev')) {
    debugPrint('[Nucleus] WARNING: You are using a dev preview ($nucleusVersion). Do not use in production.');
  }
}

// ignore: unused_element
final _devWarningPrinted = () { _printDevWarning(); return true; }();
