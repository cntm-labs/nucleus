import 'package:flutter/material.dart';
import '../widgets/nucleus_provider.dart';
import '../theme/nucleus_theme.dart';

class NucleusSignUp extends StatefulWidget {
  final VoidCallback? onSignUp;
  final List<String> oauthProviders;
  final NucleusTheme theme;

  const NucleusSignUp({
    super.key,
    this.onSignUp,
    this.oauthProviders = const [],
    this.theme = const NucleusTheme(),
  });

  @override
  State<NucleusSignUp> createState() => _NucleusSignUpState();
}

class _NucleusSignUpState extends State<NucleusSignUp> {
  final _emailController = TextEditingController();
  final _passwordController = TextEditingController();
  final _firstNameController = TextEditingController();
  final _lastNameController = TextEditingController();
  bool _isLoading = false;
  String? _error;

  @override
  void dispose() {
    _emailController.dispose(); _passwordController.dispose();
    _firstNameController.dispose(); _lastNameController.dispose();
    super.dispose();
  }

  Future<void> _handleSignUp() async {
    setState(() { _isLoading = true; _error = null; });
    try {
      final auth = NucleusProvider.of(context);
      await auth.signUp(_emailController.text, _passwordController.text,
        firstName: _firstNameController.text.isNotEmpty ? _firstNameController.text : null,
        lastName: _lastNameController.text.isNotEmpty ? _lastNameController.text : null);
      widget.onSignUp?.call();
    } catch (e) {
      setState(() { _error = e.toString(); });
    } finally {
      if (mounted) setState(() { _isLoading = false; });
    }
  }

  @override
  Widget build(BuildContext context) {
    final t = widget.theme;
    return Card(
      color: t.backgroundColor,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(t.borderRadius)),
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(mainAxisSize: MainAxisSize.min, children: [
          Text('Create Account', style: t.titleStyle ?? TextStyle(fontSize: 24, fontWeight: FontWeight.bold, color: t.textColor)),
          const SizedBox(height: 16),
          if (_error != null) ...[
            Container(
              width: double.infinity, padding: const EdgeInsets.all(8),
              decoration: BoxDecoration(color: t.errorColor.withAlpha(25), borderRadius: BorderRadius.circular(t.borderRadius)),
              child: Text(_error!, style: TextStyle(color: t.errorColor, fontSize: 14)),
            ),
            const SizedBox(height: 12),
          ],
          ...widget.oauthProviders.map((provider) => Padding(
            padding: const EdgeInsets.only(bottom: 8),
            child: OutlinedButton(
              onPressed: _isLoading ? null : () {},
              style: t.secondaryButtonStyle,
              child: Text('Continue with ${provider[0].toUpperCase()}${provider.substring(1)}'),
            ),
          )),
          if (widget.oauthProviders.isNotEmpty) ...[
            const SizedBox(height: 8),
            Row(children: [const Expanded(child: Divider()), Padding(padding: const EdgeInsets.symmetric(horizontal: 12), child: Text('or', style: TextStyle(color: Colors.grey[500], fontSize: 13))), const Expanded(child: Divider())]),
            const SizedBox(height: 8),
          ],
          Row(children: [
            Expanded(child: TextField(controller: _firstNameController, decoration: t.inputDecoration('First Name'))),
            const SizedBox(width: 8),
            Expanded(child: TextField(controller: _lastNameController, decoration: t.inputDecoration('Last Name'))),
          ]),
          const SizedBox(height: 8),
          TextField(controller: _emailController, decoration: t.inputDecoration('Email'), keyboardType: TextInputType.emailAddress),
          const SizedBox(height: 8),
          TextField(controller: _passwordController, decoration: t.inputDecoration('Password'), obscureText: true),
          const SizedBox(height: 16),
          ElevatedButton(
            onPressed: _isLoading ? null : _handleSignUp,
            style: t.primaryButtonStyle,
            child: _isLoading ? const SizedBox(width: 20, height: 20, child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white)) : const Text('Sign Up'),
          ),
        ]),
      ),
    );
  }
}
