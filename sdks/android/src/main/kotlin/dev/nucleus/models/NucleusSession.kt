package dev.nucleus.models

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

/**
 * Represents an active Nucleus session with access and refresh tokens.
 */
@Serializable
data class NucleusSession(
    val id: String,
    @SerialName("user_id")
    val userId: String,
    @SerialName("access_token")
    val accessToken: String,
    @SerialName("refresh_token")
    val refreshToken: String? = null,
    @SerialName("expires_at")
    val expiresAt: Long,
    @SerialName("created_at")
    val createdAt: Long? = null,
    @SerialName("last_active_at")
    val lastActiveAt: Long? = null,
    val status: String = "active",
) {
    /** Returns `true` when the access token has expired (with a 30-second safety margin). */
    val isExpired: Boolean
        get() = System.currentTimeMillis() / 1000 >= expiresAt - 30
}
