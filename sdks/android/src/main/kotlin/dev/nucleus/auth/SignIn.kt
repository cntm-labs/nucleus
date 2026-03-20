package dev.nucleus.auth

import android.content.Context
import androidx.credentials.CredentialManager
import androidx.credentials.GetCredentialRequest
import androidx.credentials.GetCredentialResponse
import androidx.credentials.GetPublicKeyCredentialOption
import dev.nucleus.Nucleus
import dev.nucleus.models.NucleusUser
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put

/**
 * High-level sign-in helpers that wrap Credential Manager for passkey flows.
 */
object SignIn {

    /**
     * Start a passkey-based sign-in using the Android Credential Manager.
     *
     * @param context An Activity or Fragment context (required by Credential Manager).
     * @return The signed-in [NucleusUser].
     * @throws NucleusAuthException when the flow fails or is cancelled.
     */
    suspend fun withPasskey(context: Context): NucleusUser {
        val config = Nucleus.config
        val apiClient = Nucleus.apiClient

        // 1. Request a challenge from the Nucleus backend.
        val challenge = apiClient.getPasskeyChallenge()
            ?: throw NucleusAuthException("Failed to obtain passkey challenge from server.")

        // 2. Build the Credential Manager request.
        val credentialManager = CredentialManager.create(context)

        val publicKeyOptions = GetPublicKeyCredentialOption(
            requestJson = buildPasskeyRequestJson(
                challenge = challenge.challenge,
                rpId = challenge.rpId,
                timeout = challenge.timeout,
            ),
        )

        val request = GetCredentialRequest(listOf(publicKeyOptions))

        val response: GetCredentialResponse = try {
            credentialManager.getCredential(context, request)
        } catch (e: Exception) {
            throw NucleusAuthException("Passkey authentication cancelled or failed.", e)
        }

        // 3. Send the credential to the backend for verification.
        val authResult = apiClient.verifyPasskeyCredential(response.credential)
            ?: throw NucleusAuthException("Server rejected passkey credential.")

        Nucleus.auth.setSessionFromOAuth(authResult.session, authResult.user)
        return authResult.user
    }

    /**
     * Sign in with email and password (convenience wrapper).
     */
    suspend fun withEmailPassword(email: String, password: String): NucleusUser {
        return Nucleus.auth.signInWithEmailPassword(email, password)
    }

    // ------------------------------------------------------------------
    // Private helpers
    // ------------------------------------------------------------------

    private fun buildPasskeyRequestJson(
        challenge: String,
        rpId: String,
        timeout: Long,
    ): String {
        val json = buildJsonObject {
            put("challenge", challenge)
            put("rpId", rpId)
            put("timeout", timeout)
            put("userVerification", "preferred")
        }
        return json.toString()
    }
}
