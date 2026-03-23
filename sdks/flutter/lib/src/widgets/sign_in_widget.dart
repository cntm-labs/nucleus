import 'package:flutter/material.dart';
import '../widgets/nucleus_provider.dart';
import '../theme/nucleus_theme.dart';

class NucleusSignIn extends StatefulWidget {
  final VoidCallback? onSignIn;
  final List<String> oauthProviders;
  final NucleusTheme theme;

  const NucleusSignIn({
    super.key,
    this.onSignIn,
    this.oauthProviders = const [],
    this.theme = const NucleusTheme(),
  });

  @override
  State<NucleusSignIn> createState() => _NucleusSignInState();
}

class _NucleusSignInState extends State<NucleusSignIn> {
  final _emailController = TextEditingController();
  final _passwordController = TextEditingController();
  bool _isLoading = false;
  String? _error;

  @override
  void dispose() { _emailController.dispose(); _passwordController.dispose(); super.dispose(); }

  Future<void> _handleSignIn() async {
    setState(() { _isLoading = true; _error = null; });
    try {
      final auth = NucleusProvider.of(context);
      await auth.signIn(_emailController.text, _passwordController.text);
      widget.onSignIn?.call();
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
          Text('Sign In', style: t.titleStyle ?? TextStyle(fontSize: 24, fontWeight: FontWeight.bold, color: t.textColor)),
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
              onPressed: _isLoading ? null : () { /* OAuth handled via Nucleus.auth */ },
              style: t.secondaryButtonStyle,
              child: Text('Continue with ${provider[0].toUpperCase()}${provider.substring(1)}'),
            ),
          )),
          if (widget.oauthProviders.isNotEmpty) ...[
            const SizedBox(height: 8),
            Row(children: [
              const Expanded(child: Divider()), Padding(padding: const EdgeInsets.symmetric(horizontal: 12), child: Text('or', style: TextStyle(color: Colors.grey[500], fontSize: 13))),
              const Expanded(child: Divider()),
            ]),
            const SizedBox(height: 8),
          ],
          TextField(controller: _emailController, decoration: t.inputDecoration('Email'), keyboardType: TextInputType.emailAddress),
          const SizedBox(height: 8),
          TextField(controller: _passwordController, decoration: t.inputDecoration('Password'), obscureText: true),
          const SizedBox(height: 16),
          ElevatedButton(
            onPressed: _isLoading ? null : _handleSignIn,
            style: t.primaryButtonStyle,
            child: _isLoading ? const SizedBox(width: 20, height: 20, child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white)) : const Text('Sign In'),
          ),
        ]),
      ),
    );
  }
}
