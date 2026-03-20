package dev.nucleus

import android.content.Context
import android.util.Log
import dev.nucleus.auth.NucleusAuth
import dev.nucleus.network.ApiClient
import dev.nucleus.session.SessionManager
import dev.nucleus.session.TokenStorage

/**
 * Main entry point for the Nucleus Android SDK.
 *
 * Initialize once in your Application class:
 * ```kotlin
 * Nucleus.configure(
 *     context = applicationContext,
 *     publishableKey = "pk_live_...",
 * )
 * ```
 */
object Nucleus {

    @Volatile
    private var _config: NucleusConfig? = null
    val config: NucleusConfig
        get() = _config ?: error("Nucleus has not been configured. Call Nucleus.configure() first.")

    @Volatile
    private var _auth: NucleusAuth? = null
    val auth: NucleusAuth
        get() = _auth ?: error("Nucleus has not been configured. Call Nucleus.configure() first.")

    @Volatile
    private var _apiClient: ApiClient? = null
    val apiClient: ApiClient
        get() = _apiClient ?: error("Nucleus has not been configured. Call Nucleus.configure() first.")

    @Volatile
    private var _sessionManager: SessionManager? = null
    val sessionManager: SessionManager
        get() = _sessionManager ?: error("Nucleus has not been configured. Call Nucleus.configure() first.")

    @Volatile
    private var _tokenStorage: TokenStorage? = null
    val tokenStorage: TokenStorage
        get() = _tokenStorage ?: error("Nucleus has not been configured. Call Nucleus.configure() first.")

    val isConfigured: Boolean
        get() = _config != null

    /**
     * Configure the Nucleus SDK. Must be called before any other Nucleus API.
     *
     * @param context       Application context.
     * @param publishableKey Your Nucleus publishable key (starts with `pk_`).
     * @param baseUrl       Override the API base URL (useful for self-hosted instances).
     */
    @JvmStatic
    @Synchronized
    fun configure(
        context: Context,
        publishableKey: String,
        baseUrl: String = "https://api.nucleus.dev",
    ) {
        val version = "0.1.0-dev.1"
        if ("dev" in version) {
            Log.w("Nucleus", "WARNING: You are using a dev preview ($version). Do not use in production.")
        }

        require(publishableKey.startsWith("pk_")) {
            "publishableKey must start with \"pk_\". Received: ${publishableKey.take(6)}..."
        }

        val appContext = context.applicationContext
        val cfg = NucleusConfig(
            publishableKey = publishableKey,
            baseUrl = baseUrl.trimEnd('/'),
        )
        _config = cfg

        val tokenStorage = TokenStorage(appContext)
        _tokenStorage = tokenStorage

        val apiClient = ApiClient(cfg, tokenStorage)
        _apiClient = apiClient

        val sessionManager = SessionManager(apiClient, tokenStorage)
        _sessionManager = sessionManager

        val auth = NucleusAuth(apiClient, tokenStorage, sessionManager)
        _auth = auth

        // Attempt to restore a previous session on startup.
        sessionManager.restoreSessionIfAvailable()
    }

    /**
     * Tear down the SDK — useful for tests or logout-and-reconfigure flows.
     */
    @JvmStatic
    @Synchronized
    fun reset() {
        _sessionManager?.stop()
        _config = null
        _auth = null
        _apiClient = null
        _sessionManager = null
        _tokenStorage = null
    }
}

/**
 * Immutable configuration snapshot.
 */
data class NucleusConfig(
    val publishableKey: String,
    val baseUrl: String,
)
