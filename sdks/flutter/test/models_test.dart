import 'package:flutter_test/flutter_test.dart';
import 'package:cntm_nucleus/nucleus.dart';

void main() {
  group('NucleusUser', () {
    test('fromJson parses all fields', () {
      final json = {
        'id': 'user_1', 'email': 'test@example.com',
        'email_verified': true, 'first_name': 'John', 'last_name': 'Doe',
        'avatar_url': 'https://example.com/avatar.jpg',
        'metadata': {'key': 'value'}, 'created_at': '2024-01-01T00:00:00Z',
      };
      final user = NucleusUser.fromJson(json);
      expect(user.id, 'user_1');
      expect(user.email, 'test@example.com');
      expect(user.emailVerified, true);
      expect(user.firstName, 'John');
      expect(user.lastName, 'Doe');
      expect(user.fullName, 'John Doe');
    });

    test('fromJson handles missing optional fields gracefully', () {
      final json = <String, dynamic>{'id': 'u_1', 'email': 'a@b.com', 'created_at': '2024-01-01T00:00:00Z', 'metadata': <String, dynamic>{}};
      final user = NucleusUser.fromJson(json);
      expect(user.email, 'a@b.com');
      expect(user.id, 'u_1');
    });

    test('fullName with both names', () {
      final json = <String, dynamic>{'id': 'u_1', 'email': 'a@b.com', 'first_name': 'John', 'last_name': 'Doe', 'metadata': <String, dynamic>{}, 'created_at': '2024-01-01T00:00:00Z'};
      final user = NucleusUser.fromJson(json);
      expect(user.fullName, 'John Doe');
    });
  });

  group('NucleusSession', () {
    test('fromJson parses all fields', () {
      final json = {'id': 's_1', 'token': 'tok', 'refresh_token': 'ref', 'expires_at': '2024-12-31T23:59:59Z', 'user_id': 'u_1'};
      final session = NucleusSession.fromJson(json);
      expect(session.id, 's_1');
      expect(session.token, 'tok');
      expect(session.refreshToken, 'ref');
      expect(session.userId, 'u_1');
    });

    test('isExpired returns true for past dates', () {
      final session = NucleusSession(id: 's', token: 't', refreshToken: 'r', expiresAt: DateTime(2020));
      expect(session.isExpired, true);
    });

    test('isExpired returns false for future dates', () {
      final session = NucleusSession(id: 's', token: 't', refreshToken: 'r', expiresAt: DateTime(2030));
      expect(session.isExpired, false);
    });
  });

  group('NucleusOrganization', () {
    test('fromJson parses all fields', () {
      final json = {'id': 'org_1', 'name': 'Test Org', 'slug': 'test-org', 'created_at': '2024-01-01T00:00:00Z'};
      final org = NucleusOrganization.fromJson(json);
      expect(org.id, 'org_1');
      expect(org.name, 'Test Org');
      expect(org.slug, 'test-org');
    });
  });

  group('NucleusMember', () {
    test('fromJson parses permissions list', () {
      final json = {'id': 'm_1', 'user_id': 'u_1', 'org_id': 'o_1', 'role': 'admin', 'email': 'a@b.com', 'permissions': ['read', 'write']};
      final member = NucleusMember.fromJson(json);
      expect(member.permissions, ['read', 'write']);
      expect(member.role, 'admin');
    });

    test('fromJson handles missing permissions', () {
      final json = {'id': 'm_1', 'user_id': 'u_1', 'org_id': 'o_1', 'role': 'member', 'email': 'a@b.com'};
      final member = NucleusMember.fromJson(json);
      expect(member.permissions, isEmpty);
    });
  });

  group('NucleusInvitation', () {
    test('fromJson parses all fields', () {
      final json = {'id': 'i_1', 'org_id': 'o_1', 'email': 'a@b.com', 'role': 'member', 'status': 'pending', 'created_at': '2024-01-01T00:00:00Z'};
      final inv = NucleusInvitation.fromJson(json);
      expect(inv.id, 'i_1');
      expect(inv.status, 'pending');
      expect(inv.role, 'member');
    });
  });

  group('NucleusConfig', () {
    test('effectiveBaseUrl uses default when no apiBaseUrl', () {
      final config = NucleusConfig(publishableKey: 'pk_test');
      expect(config.effectiveBaseUrl, 'https://api.nucleus.dev');
    });

    test('effectiveBaseUrl uses custom when provided', () {
      final config = NucleusConfig(publishableKey: 'pk_test', apiBaseUrl: 'https://custom.api.com');
      expect(config.effectiveBaseUrl, 'https://custom.api.com');
    });
  });
}
