package dev.nucleus.models

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

/**
 * Represents an authenticated Nucleus user.
 */
@Serializable
data class NucleusUser(
    val id: String,
    @SerialName("email_address")
    val emailAddress: String? = null,
    @SerialName("first_name")
    val firstName: String? = null,
    @SerialName("last_name")
    val lastName: String? = null,
    @SerialName("image_url")
    val imageUrl: String? = null,
    @SerialName("created_at")
    val createdAt: Long? = null,
    @SerialName("updated_at")
    val updatedAt: Long? = null,
    @SerialName("external_accounts")
    val externalAccounts: List<ExternalAccount> = emptyList(),
) {
    val fullName: String?
        get() = listOfNotNull(firstName, lastName).takeIf { it.isNotEmpty() }?.joinToString(" ")
}

@Serializable
data class ExternalAccount(
    val id: String,
    val provider: String,
    @SerialName("provider_user_id")
    val providerUserId: String? = null,
    @SerialName("email_address")
    val emailAddress: String? = null,
)
