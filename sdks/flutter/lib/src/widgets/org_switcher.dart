import 'package:flutter/material.dart';
import '../widgets/nucleus_provider.dart';
import '../models/organization.dart';

class NucleusOrgSwitcher extends StatefulWidget {
  final VoidCallback? onSelect;

  const NucleusOrgSwitcher({super.key, this.onSelect});

  @override
  State<NucleusOrgSwitcher> createState() => _NucleusOrgSwitcherState();
}

class _NucleusOrgSwitcherState extends State<NucleusOrgSwitcher> {
  List<NucleusOrganization> _organizations = [];
  bool _isLoading = false;

  Future<void> _loadOrgs() async {
    setState(() { _isLoading = true; });
    try {
      final auth = NucleusProvider.of(context);
      _organizations = await auth.getOrganizations();
    } catch (_) {}
    finally { if (mounted) setState(() { _isLoading = false; }); }
  }

  @override
  Widget build(BuildContext context) {
    final auth = NucleusProvider.of(context);
    return PopupMenuButton<NucleusOrganization>(
      offset: const Offset(0, 44),
      onOpened: _loadOrgs,
      onSelected: (org) {
        auth.setOrganization(org);
        widget.onSelect?.call();
      },
      itemBuilder: (context) {
        if (_isLoading) return [const PopupMenuItem(enabled: false, child: Center(child: CircularProgressIndicator()))];
        if (_organizations.isEmpty) return [const PopupMenuItem(enabled: false, child: Text('No organizations'))];
        return _organizations.map((org) => PopupMenuItem(
          value: org,
          child: Row(children: [
            if (auth.organization?.id == org.id) const Icon(Icons.check, size: 16) else const SizedBox(width: 16),
            const SizedBox(width: 8),
            Text(org.name),
          ]),
        )).toList();
      },
      child: OutlinedButton.icon(
        onPressed: null,
        icon: const Icon(Icons.business, size: 16),
        label: Text(auth.organization?.name ?? 'Select Organization'),
      ),
    );
  }
}
