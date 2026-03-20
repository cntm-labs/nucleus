import Foundation

/// Represents an active authentication session with access and refresh tokens.
public struct NucleusSession: Codable, Equatable, Sendable {
    public let accessToken: String
    public let refreshToken: String
    public let expiresAt: Date
    public let tokenType: String?

    /// Whether the access token has expired.
    public var isExpired: Bool {
        Date() >= expiresAt
    }

    /// Whether the access token will expire within 60 seconds.
    public var isExpiringSoon: Bool {
        Date().addingTimeInterval(60) >= expiresAt
    }

    enum CodingKeys: String, CodingKey {
        case accessToken = "access_token"
        case refreshToken = "refresh_token"
        case expiresAt = "expires_at"
        case tokenType = "token_type"
    }
}
