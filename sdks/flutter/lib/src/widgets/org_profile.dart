import 'package:flutter/material.dart';
import '../widgets/nucleus_provider.dart';
import '../models/member.dart';
import '../theme/nucleus_theme.dart';

class NucleusOrgProfile extends StatefulWidget {
  final NucleusTheme theme;

  const NucleusOrgProfile({super.key, this.theme = const NucleusTheme()});

  @override
  State<NucleusOrgProfile> createState() => _NucleusOrgProfileState();
}

class _NucleusOrgProfileState extends State<NucleusOrgProfile> {
  List<NucleusMember> _members = [];
  final _emailController = TextEditingController();
  String _inviteRole = 'member';
  bool _isLoading = false;
  String? _error;

  @override
  void initState() {
    super.initState();
    _loadMembers();
  }

  @override
  void dispose() { _emailController.dispose(); super.dispose(); }

  Future<void> _loadMembers() async {
    final auth = NucleusProvider.of(context);
    if (auth.organization == null) return;
    try {
      _members = await auth.getMembers(auth.organization!.id);
      if (mounted) setState(() {});
    } catch (_) {}
  }

  Future<void> _invite() async {
    final auth = NucleusProvider.of(context);
    if (auth.organization == null) return;
    setState(() { _isLoading = true; _error = null; });
    try {
      await auth.inviteMember(auth.organization!.id, _emailController.text, _inviteRole);
      _emailController.clear();
    } catch (e) { setState(() { _error = e.toString(); }); }
    finally { if (mounted) setState(() { _isLoading = false; }); }
  }

  @override
  Widget build(BuildContext context) {
    final t = widget.theme;
    final auth = NucleusProvider.of(context);
    final org = auth.organization;

    if (org == null) {
      return Card(
        color: t.backgroundColor,
        child: const Padding(padding: EdgeInsets.all(24), child: Text('No organization selected')),
      );
    }

    return Card(
      color: t.backgroundColor,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(t.borderRadius)),
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
          Center(child: Text(org.name, style: t.titleStyle ?? TextStyle(fontSize: 24, fontWeight: FontWeight.bold, color: t.textColor))),
          Center(child: Text(org.slug, style: TextStyle(color: Colors.grey[600], fontSize: 13))),
          const SizedBox(height: 16),
          if (_error != null) Container(width: double.infinity, padding: const EdgeInsets.all(8), margin: const EdgeInsets.only(bottom: 12),
            decoration: BoxDecoration(color: t.errorColor.withAlpha(25), borderRadius: BorderRadius.circular(t.borderRadius)),
            child: Text(_error!, style: TextStyle(color: t.errorColor, fontSize: 14))),
          Text('Members', style: TextStyle(fontSize: 16, fontWeight: FontWeight.w600, color: t.textColor)),
          const SizedBox(height: 8),
          ..._members.map((m) => ListTile(
            dense: true, contentPadding: EdgeInsets.zero,
            title: Text('${m.firstName ?? ''} ${m.lastName ?? ''}'.trim()),
            subtitle: Text('${m.email} · ${m.role}'),
            trailing: IconButton(icon: const Icon(Icons.remove_circle_outline, color: Colors.red, size: 20), onPressed: () async {
              try { await auth.removeMember(org.id, m.id); _loadMembers(); } catch (e) { setState(() { _error = e.toString(); }); }
            }),
          )),
          const SizedBox(height: 16),
          Text('Invite Member', style: TextStyle(fontSize: 16, fontWeight: FontWeight.w600, color: t.textColor)),
          const SizedBox(height: 8),
          TextField(controller: _emailController, decoration: t.inputDecoration('Email'), keyboardType: TextInputType.emailAddress),
          const SizedBox(height: 8),
          DropdownButtonFormField<String>(
            initialValue: _inviteRole,
            decoration: t.inputDecoration('Role'),
            items: const [DropdownMenuItem(value: 'member', child: Text('Member')), DropdownMenuItem(value: 'admin', child: Text('Admin'))],
            onChanged: (v) { if (v != null) setState(() { _inviteRole = v; }); },
          ),
          const SizedBox(height: 16),
          ElevatedButton(onPressed: _isLoading ? null : _invite, style: t.primaryButtonStyle,
            child: _isLoading ? const SizedBox(width: 20, height: 20, child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white)) : const Text('Send Invitation')),
        ]),
      ),
    );
  }
}
