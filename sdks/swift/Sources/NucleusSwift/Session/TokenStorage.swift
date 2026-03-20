import Foundation
import Security

/// Thin wrapper around the iOS Keychain for persisting tokens.
///
/// Items are stored with `kSecAttrAccessibleAfterFirstUnlock` so they remain
/// available across app launches and while the device is locked (after the
/// first unlock).
final class TokenStorage: Sendable {

    static let shared = TokenStorage()

    private let service = "com.nucleus.sdk"

    private init() {}

    // MARK: - Public API

    /// Save arbitrary `Data` to the Keychain under the given key.
    func save(_ data: Data, forKey key: String) {
        // Delete any existing item first to avoid `errSecDuplicateItem`.
        delete(forKey: key)

        let query: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrService: service,
            kSecAttrAccount: key,
            kSecValueData: data,
            kSecAttrAccessible: kSecAttrAccessibleAfterFirstUnlock,
        ]

        let status = SecItemAdd(query as CFDictionary, nil)
        if status != errSecSuccess {
            logKeychainError("save", key: key, status: status)
        }
    }

    /// Load data previously stored under the given key.
    func load(forKey key: String) -> Data? {
        let query: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrService: service,
            kSecAttrAccount: key,
            kSecReturnData: true,
            kSecMatchLimit: kSecMatchLimitOne,
        ]

        var result: AnyObject?
        let status = SecItemCopyMatching(query as CFDictionary, &result)

        guard status == errSecSuccess else {
            if status != errSecItemNotFound {
                logKeychainError("load", key: key, status: status)
            }
            return nil
        }

        return result as? Data
    }

    /// Delete the item stored under the given key.
    @discardableResult
    func delete(forKey key: String) -> Bool {
        let query: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrService: service,
            kSecAttrAccount: key,
        ]

        let status = SecItemDelete(query as CFDictionary)
        if status != errSecSuccess && status != errSecItemNotFound {
            logKeychainError("delete", key: key, status: status)
            return false
        }
        return true
    }

    // MARK: - Private

    private func logKeychainError(_ operation: String, key: String, status: OSStatus) {
        #if DEBUG
        print("[NucleusSwift] Keychain \(operation) failed for \"\(key)\": OSStatus \(status)")
        #endif
    }
}
