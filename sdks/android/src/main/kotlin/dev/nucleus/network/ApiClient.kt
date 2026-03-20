package dev.nucleus.network

import androidx.credentials.Credential
import dev.nucleus.NucleusConfig
import dev.nucleus.models.NucleusOrganization
import dev.nucleus.models.NucleusSession
import dev.nucleus.models.NucleusUser
import dev.nucleus.session.SessionRestoreResponse
import dev.nucleus.session.TokenStorage
import kotlinx.serialization.json.Json
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import okhttp3.logging.HttpLoggingInterceptor
import java.util.concurrent.TimeUnit

/**
 * Low-level HTTP client that communicates with the Nucleus API.
 *
 * All public methods are `suspend` so they can be called from coroutines
 * without blocking the main thread.
 */
class ApiClient internal constructor(
    private val config: NucleusConfig,
    private val tokenStorage: TokenStorage,
) {

    private val json = Json {
        ignoreUnknownKeys = true
        encodeDefaults = true
        isLenient = true
    }

    private val jsonMediaType = "application/json; charset=utf-8".toMediaType()

    val httpClient: OkHttpClient = OkHttpClient.Builder()
        .addInterceptor(NucleusAuthInterceptor(config, tokenStorage))
        .addInterceptor(HttpLoggingInterceptor().apply {
            level = HttpLoggingInterceptor.Level.BODY
        })
        .connectTimeout(30, TimeUnit.SECONDS)
        .readTimeout(30, TimeUnit.SECONDS)
        .writeTimeout(30, TimeUnit.SECONDS)
        .build()

    // ------------------------------------------------------------------
    // Auth endpoints
    // ------------------------------------------------------------------

    data class AuthResult(val session: NucleusSession, val user: NucleusUser)

    suspend fun signIn(email: String, password: String): AuthResult? {
        val body = json.encodeToString(
            kotlinx.serialization.serializer<Map<String, String>>(),
            mapOf("email" to email, "password" to password),
        )
        val request = newRequest("v1/auth/sign-in")
            .post(body.toRequestBody(jsonMediaType))
            .build()
        return executeForAuthResult(request)
    }

    suspend fun signOut() {
        val request = newRequest("v1/auth/sign-out")
            .post("{}".toRequestBody(jsonMediaType))
            .build()
        httpClient.newCall(request).execute().close()
    }

    // ------------------------------------------------------------------
    // Passkey endpoints
    // ------------------------------------------------------------------

    data class PasskeyChallenge(val challenge: String, val rpId: String, val timeout: Long)

    suspend fun getPasskeyChallenge(): PasskeyChallenge? {
        val request = newRequest("v1/auth/passkey/challenge")
            .post("{}".toRequestBody(jsonMediaType))
            .build()
        val response = httpClient.newCall(request).execute()
        if (!response.isSuccessful) return null
        val responseBody = response.body?.string() ?: return null
        val obj = json.decodeFromString<Map<String, kotlinx.serialization.json.JsonElement>>(responseBody)
        return PasskeyChallenge(
            challenge = obj["challenge"].toString().trim('"'),
            rpId = obj["rp_id"].toString().trim('"'),
            timeout = obj["timeout"].toString().trim('"').toLongOrNull() ?: 60000,
        )
    }

    suspend fun verifyPasskeyCredential(credential: Credential): AuthResult? {
        val body = json.encodeToString(
            kotlinx.serialization.serializer<Map<String, String>>(),
            mapOf("credential_type" to credential.type, "data" to credential.data.toString()),
        )
        val request = newRequest("v1/auth/passkey/verify")
            .post(body.toRequestBody(jsonMediaType))
            .build()
        return executeForAuthResult(request)
    }

    // ------------------------------------------------------------------
    // OAuth endpoints
    // ------------------------------------------------------------------

    suspend fun exchangeOAuthCode(code: String, redirectUri: String): AuthResult? {
        val body = json.encodeToString(
            kotlinx.serialization.serializer<Map<String, String>>(),
            mapOf("code" to code, "redirect_uri" to redirectUri),
        )
        val request = newRequest("v1/oauth/token")
            .post(body.toRequestBody(jsonMediaType))
            .build()
        return executeForAuthResult(request)
    }

    // ------------------------------------------------------------------
    // Session endpoints
    // ------------------------------------------------------------------

    suspend fun getSession(sessionId: String): SessionRestoreResponse? {
        val request = newRequest("v1/sessions/$sessionId").get().build()
        val response = httpClient.newCall(request).execute()
        if (!response.isSuccessful) return null
        val responseBody = response.body?.string() ?: return null
        val session = json.decodeFromString<NucleusSession>(responseBody)
        // Fetch user separately.
        val user = getUser(session.userId)
        return SessionRestoreResponse(session, user)
    }

    suspend fun refreshSession(refreshToken: String): NucleusSession? {
        val body = json.encodeToString(
            kotlinx.serialization.serializer<Map<String, String>>(),
            mapOf("refresh_token" to refreshToken),
        )
        val request = newRequest("v1/sessions/refresh")
            .post(body.toRequestBody(jsonMediaType))
            .build()
        val response = httpClient.newCall(request).execute()
        if (!response.isSuccessful) return null
        val responseBody = response.body?.string() ?: return null
        return json.decodeFromString<NucleusSession>(responseBody)
    }

    // ------------------------------------------------------------------
    // User endpoints
    // ------------------------------------------------------------------

    suspend fun getUser(userId: String): NucleusUser? {
        val request = newRequest("v1/users/$userId").get().build()
        val response = httpClient.newCall(request).execute()
        if (!response.isSuccessful) return null
        val responseBody = response.body?.string() ?: return null
        return json.decodeFromString<NucleusUser>(responseBody)
    }

    // ------------------------------------------------------------------
    // Organization endpoints
    // ------------------------------------------------------------------

    suspend fun getOrganizations(): List<NucleusOrganization> {
        val request = newRequest("v1/organizations").get().build()
        val response = httpClient.newCall(request).execute()
        if (!response.isSuccessful) return emptyList()
        val responseBody = response.body?.string() ?: return emptyList()
        return json.decodeFromString<List<NucleusOrganization>>(responseBody)
    }

    // ------------------------------------------------------------------
    // Helpers
    // ------------------------------------------------------------------

    private fun newRequest(path: String): Request.Builder {
        return Request.Builder().url("${config.baseUrl}/$path")
    }

    private fun executeForAuthResult(request: Request): AuthResult? {
        val response = httpClient.newCall(request).execute()
        if (!response.isSuccessful) return null
        val responseBody = response.body?.string() ?: return null
        // Expect { "session": { ... }, "user": { ... } }
        val element = json.parseToJsonElement(responseBody)
        val sessionJson = element.jsonObject["session"] ?: return null
        val userJson = element.jsonObject["user"] ?: return null
        val session = json.decodeFromJsonElement(NucleusSession.serializer(), sessionJson)
        val user = json.decodeFromJsonElement(NucleusUser.serializer(), userJson)
        return AuthResult(session, user)
    }

    private val kotlinx.serialization.json.JsonElement.jsonObject
        get() = this as kotlinx.serialization.json.JsonObject
}
