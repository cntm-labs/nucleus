import 'package:flutter/material.dart';
import '../widgets/nucleus_provider.dart';
import '../nucleus.dart';

class NucleusUserButton extends StatelessWidget {
  final VoidCallback? onSignOut;

  const NucleusUserButton({super.key, this.onSignOut});

  @override
  Widget build(BuildContext context) {
    final auth = NucleusProvider.of(context);
    final user = auth.user;
    if (user == null) return const SizedBox.shrink();

    final initials = [user.firstName, user.lastName]
      .whereType<String>().where((s) => s.isNotEmpty)
      .map((s) => s[0]).join().toUpperCase();
    final display = initials.isNotEmpty ? initials : user.email[0].toUpperCase();

    return PopupMenuButton<String>(
      offset: const Offset(0, 44),
      onSelected: (value) async {
        if (value == 'signout') {
          await Nucleus.signOut();
          onSignOut?.call();
        }
      },
      itemBuilder: (context) => [
        PopupMenuItem(enabled: false, child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
          Text(user.fullName, style: const TextStyle(fontWeight: FontWeight.w600, fontSize: 14, color: Colors.black)),
          Text(user.email, style: TextStyle(fontSize: 12, color: Colors.grey[600])),
        ])),
        const PopupMenuDivider(),
        const PopupMenuItem(value: 'signout', child: Text('Sign Out', style: TextStyle(color: Colors.red))),
      ],
      child: CircleAvatar(
        radius: 18,
        backgroundColor: const Color(0xFF4C6EF5),
        backgroundImage: user.avatarUrl != null ? NetworkImage(user.avatarUrl!) : null,
        child: user.avatarUrl == null ? Text(display, style: const TextStyle(color: Colors.white, fontSize: 14, fontWeight: FontWeight.bold)) : null,
      ),
    );
  }
}
