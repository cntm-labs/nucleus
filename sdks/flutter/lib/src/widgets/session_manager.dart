import 'package:flutter/material.dart';
import '../models/active_session.dart';
import '../nucleus.dart';

/// Displays the user's active sessions and allows revoking any of them.
///
/// Mount this widget in a "Security" or "Account" settings screen. It fetches
/// the session list automatically and shows device name, IP address, and
/// last-active timestamp. Tapping "Sign out" on a session calls the remote-
/// logout endpoint, removing that device's access.
///
/// ```dart
/// // Basic usage:
/// const NucleusSessionManager()
///
/// // With custom title and theming:
/// NucleusSessionManager(
///   title: 'Active devices',
///   onSessionRevoked: (id) => print('Revoked $id'),
/// )
/// ```
class NucleusSessionManager extends StatefulWidget {
  final String title;
  final void Function(String sessionId)? onSessionRevoked;

  const NucleusSessionManager({
    super.key,
    this.title = 'Active Sessions',
    this.onSessionRevoked,
  });

  @override
  State<NucleusSessionManager> createState() => _NucleusSessionManagerState();
}

class _NucleusSessionManagerState extends State<NucleusSessionManager> {
  List<NucleusActiveSession>? _sessions;
  String? _error;
  final Set<String> _revoking = {};

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    setState(() {
      _sessions = null;
      _error = null;
    });
    try {
      final sessions = await Nucleus.listActiveSessions();
      if (mounted) setState(() => _sessions = sessions);
    } catch (e) {
      if (mounted) setState(() => _error = e.toString());
    }
  }

  Future<void> _revoke(String sessionId) async {
    setState(() => _revoking.add(sessionId));
    try {
      await Nucleus.revokeSession(sessionId);
      widget.onSessionRevoked?.call(sessionId);
      if (mounted) {
        setState(() {
          _sessions?.removeWhere((s) => s.id == sessionId);
          _revoking.remove(sessionId);
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() => _revoking.remove(sessionId));
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Failed to revoke session: $e')),
        );
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
          child: Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(widget.title,
                  style: Theme.of(context).textTheme.titleMedium?.copyWith(fontWeight: FontWeight.w600)),
              if (_sessions != null)
                TextButton(onPressed: _load, child: const Text('Refresh')),
            ],
          ),
        ),
        if (_error != null)
          _ErrorView(error: _error!, onRetry: _load)
        else if (_sessions == null)
          const Center(child: Padding(padding: EdgeInsets.all(32), child: CircularProgressIndicator()))
        else if (_sessions!.isEmpty)
          const Padding(
            padding: EdgeInsets.all(32),
            child: Center(child: Text('No active sessions found.')),
          )
        else
          ListView.separated(
            shrinkWrap: true,
            physics: const NeverScrollableScrollPhysics(),
            itemCount: _sessions!.length,
            separatorBuilder: (_, __) => const Divider(height: 1),
            itemBuilder: (context, i) => _SessionTile(
              session: _sessions![i],
              isRevoking: _revoking.contains(_sessions![i].id),
              onRevoke: () => _revoke(_sessions![i].id),
            ),
          ),
      ],
    );
  }
}

class _SessionTile extends StatelessWidget {
  final NucleusActiveSession session;
  final bool isRevoking;
  final VoidCallback onRevoke;

  const _SessionTile({
    required this.session,
    required this.isRevoking,
    required this.onRevoke,
  });

  @override
  Widget build(BuildContext context) {
    final label = session.deviceName ?? session.deviceType ?? 'Unknown device';
    final detail = [
      if (session.browser != null) session.browser!,
      if (session.ip != null) session.ip!,
      if (session.countryCode != null) session.countryCode!,
    ].join(' · ');

    final lastActive = _formatRelative(session.lastActiveAt);

    return ListTile(
      leading: CircleAvatar(
        backgroundColor: Theme.of(context).colorScheme.surfaceContainerHighest,
        child: Icon(_deviceIcon(session.deviceType),
            color: Theme.of(context).colorScheme.onSurfaceVariant),
      ),
      title: Text(label, style: const TextStyle(fontWeight: FontWeight.w500)),
      subtitle: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          if (detail.isNotEmpty) Text(detail, style: const TextStyle(fontSize: 12)),
          Text('Last active $lastActive', style: const TextStyle(fontSize: 12)),
        ],
      ),
      isThreeLine: detail.isNotEmpty,
      trailing: isRevoking
          ? const SizedBox(width: 20, height: 20, child: CircularProgressIndicator(strokeWidth: 2))
          : TextButton(
              onPressed: onRevoke,
              child: Text('Sign out',
                  style: TextStyle(color: Theme.of(context).colorScheme.error)),
            ),
    );
  }

  IconData _deviceIcon(String? deviceType) {
    switch (deviceType?.toLowerCase()) {
      case 'mobile':
      case 'phone':
        return Icons.phone_android;
      case 'tablet':
        return Icons.tablet_android;
      case 'desktop':
      case 'web':
        return Icons.computer;
      default:
        return Icons.devices;
    }
  }

  String _formatRelative(DateTime dt) {
    final diff = DateTime.now().difference(dt);
    if (diff.inMinutes < 1) return 'just now';
    if (diff.inHours < 1) return '${diff.inMinutes}m ago';
    if (diff.inDays < 1) return '${diff.inHours}h ago';
    if (diff.inDays < 30) return '${diff.inDays}d ago';
    return dt.toLocal().toString().substring(0, 10);
  }
}

class _ErrorView extends StatelessWidget {
  final String error;
  final VoidCallback onRetry;

  const _ErrorView({required this.error, required this.onRetry});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(32),
      child: Column(
        children: [
          Icon(Icons.error_outline, color: Theme.of(context).colorScheme.error, size: 40),
          const SizedBox(height: 8),
          Text('Failed to load sessions', style: Theme.of(context).textTheme.bodyMedium),
          const SizedBox(height: 12),
          OutlinedButton(onPressed: onRetry, child: const Text('Retry')),
        ],
      ),
    );
  }
}
