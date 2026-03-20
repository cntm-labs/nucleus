import Foundation

/// Represents an authenticated Nucleus user.
public struct NucleusUser: Codable, Identifiable, Equatable, Sendable {
    public let id: String
    public let email: String
    public let firstName: String?
    public let lastName: String?
    public let avatarURL: String?
    public let emailVerified: Bool?
    public let createdAt: Date?
    public let updatedAt: Date?

    public var displayName: String {
        [firstName, lastName]
            .compactMap { $0 }
            .joined(separator: " ")
            .trimmingCharacters(in: .whitespaces)
            .isEmpty
            ? email
            : [firstName, lastName].compactMap { $0 }.joined(separator: " ")
    }

    public var initials: String {
        let parts = displayName.split(separator: " ")
        let letters = parts.prefix(2).compactMap { $0.first }
        return String(letters).uppercased()
    }

    enum CodingKeys: String, CodingKey {
        case id
        case email
        case firstName = "first_name"
        case lastName = "last_name"
        case avatarURL = "avatar_url"
        case emailVerified = "email_verified"
        case createdAt = "created_at"
        case updatedAt = "updated_at"
    }
}
