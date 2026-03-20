package dev.nucleus.models

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

/**
 * Represents a Nucleus organization the user belongs to.
 */
@Serializable
data class NucleusOrganization(
    val id: String,
    val name: String,
    val slug: String? = null,
    @SerialName("image_url")
    val imageUrl: String? = null,
    @SerialName("created_at")
    val createdAt: Long? = null,
    @SerialName("updated_at")
    val updatedAt: Long? = null,
    val role: String? = null,
    @SerialName("members_count")
    val membersCount: Int? = null,
)
