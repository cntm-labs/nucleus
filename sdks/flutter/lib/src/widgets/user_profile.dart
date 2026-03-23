import 'package:flutter/material.dart';
import '../widgets/nucleus_provider.dart';
import '../theme/nucleus_theme.dart';

class NucleusUserProfile extends StatefulWidget {
  final VoidCallback? onUpdate;
  final NucleusTheme theme;

  const NucleusUserProfile({super.key, this.onUpdate, this.theme = const NucleusTheme()});

  @override
  State<NucleusUserProfile> createState() => _NucleusUserProfileState();
}

class _NucleusUserProfileState extends State<NucleusUserProfile> {
  late TextEditingController _firstNameController;
  late TextEditingController _lastNameController;
  final _currentPasswordController = TextEditingController();
  final _newPasswordController = TextEditingController();
  bool _isLoading = false;
  String? _error, _success;
  int _tab = 0;

  @override
  void initState() {
    super.initState();
    final auth = NucleusProvider.of(context);
    _firstNameController = TextEditingController(text: auth.user?.firstName ?? '');
    _lastNameController = TextEditingController(text: auth.user?.lastName ?? '');
  }

  @override
  void dispose() {
    _firstNameController.dispose(); _lastNameController.dispose();
    _currentPasswordController.dispose(); _newPasswordController.dispose();
    super.dispose();
  }

  Future<void> _updateProfile() async {
    setState(() { _isLoading = true; _error = null; _success = null; });
    try {
      final auth = NucleusProvider.of(context);
      await auth.updateProfile(firstName: _firstNameController.text, lastName: _lastNameController.text);
      setState(() { _success = 'Profile updated'; });
      widget.onUpdate?.call();
    } catch (e) { setState(() { _error = e.toString(); }); }
    finally { if (mounted) setState(() { _isLoading = false; }); }
  }

  Future<void> _updatePassword() async {
    setState(() { _isLoading = true; _error = null; _success = null; });
    try {
      final auth = NucleusProvider.of(context);
      await auth.updatePassword(_currentPasswordController.text, _newPasswordController.text);
      setState(() { _success = 'Password updated'; _currentPasswordController.clear(); _newPasswordController.clear(); });
    } catch (e) { setState(() { _error = e.toString(); }); }
    finally { if (mounted) setState(() { _isLoading = false; }); }
  }

  @override
  Widget build(BuildContext context) {
    final t = widget.theme;
    final auth = NucleusProvider.of(context);
    if (auth.user == null) return const SizedBox.shrink();

    return Card(
      color: t.backgroundColor,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(t.borderRadius)),
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(mainAxisSize: MainAxisSize.min, children: [
          Text('Profile', style: t.titleStyle ?? TextStyle(fontSize: 24, fontWeight: FontWeight.bold, color: t.textColor)),
          const SizedBox(height: 16),
          if (_error != null) Container(width: double.infinity, padding: const EdgeInsets.all(8), margin: const EdgeInsets.only(bottom: 12),
            decoration: BoxDecoration(color: t.errorColor.withAlpha(25), borderRadius: BorderRadius.circular(t.borderRadius)),
            child: Text(_error!, style: TextStyle(color: t.errorColor, fontSize: 14))),
          if (_success != null) Container(width: double.infinity, padding: const EdgeInsets.all(8), margin: const EdgeInsets.only(bottom: 12),
            decoration: BoxDecoration(color: Colors.green.withAlpha(25), borderRadius: BorderRadius.circular(t.borderRadius)),
            child: Text(_success!, style: const TextStyle(color: Colors.green, fontSize: 14))),
          Row(children: [
            Expanded(child: OutlinedButton(onPressed: () => setState(() => _tab = 0), style: _tab == 0 ? t.primaryButtonStyle : t.secondaryButtonStyle, child: const Text('Profile'))),
            const SizedBox(width: 8),
            Expanded(child: OutlinedButton(onPressed: () => setState(() => _tab = 1), style: _tab == 1 ? t.primaryButtonStyle : t.secondaryButtonStyle, child: const Text('Password'))),
          ]),
          const SizedBox(height: 16),
          if (_tab == 0) ...[
            TextField(enabled: false, decoration: t.inputDecoration('Email').copyWith(hintText: auth.user!.email)),
            const SizedBox(height: 8),
            TextField(controller: _firstNameController, decoration: t.inputDecoration('First Name')),
            const SizedBox(height: 8),
            TextField(controller: _lastNameController, decoration: t.inputDecoration('Last Name')),
            const SizedBox(height: 16),
            ElevatedButton(onPressed: _isLoading ? null : _updateProfile, style: t.primaryButtonStyle,
              child: _isLoading ? const SizedBox(width: 20, height: 20, child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white)) : const Text('Save Changes')),
          ],
          if (_tab == 1) ...[
            TextField(controller: _currentPasswordController, decoration: t.inputDecoration('Current Password'), obscureText: true),
            const SizedBox(height: 8),
            TextField(controller: _newPasswordController, decoration: t.inputDecoration('New Password'), obscureText: true),
            const SizedBox(height: 16),
            ElevatedButton(onPressed: _isLoading ? null : _updatePassword, style: t.primaryButtonStyle,
              child: _isLoading ? const SizedBox(width: 20, height: 20, child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white)) : const Text('Update Password')),
          ],
        ]),
      ),
    );
  }
}
