import Foundation

/// Represents an organization the user belongs to.
public struct NucleusOrganization: Codable, Identifiable, Equatable, Sendable {
    public let id: String
    public let name: String
    public let slug: String?
    public let logoURL: String?
    public let createdAt: Date?
    public let updatedAt: Date?

    enum CodingKeys: String, CodingKey {
        case id
        case name
        case slug
        case logoURL = "logo_url"
        case createdAt = "created_at"
        case updatedAt = "updated_at"
    }
}
