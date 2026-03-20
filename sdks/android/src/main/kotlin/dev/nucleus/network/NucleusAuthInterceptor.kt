package dev.nucleus.network

import dev.nucleus.NucleusConfig
import dev.nucleus.session.TokenStorage
import okhttp3.Interceptor
import okhttp3.Response

/**
 * OkHttp [Interceptor] that automatically attaches authentication headers
 * to every outbound request.
 *
 * Headers added:
 * - `Authorization: Bearer <access_token>` (when a token is available)
 * - `X-Nucleus-Publishable-Key: <key>`
 * - `User-Agent: nucleus-android/<version>`
 */
class NucleusAuthInterceptor(
    private val config: NucleusConfig,
    private val tokenStorage: TokenStorage,
) : Interceptor {

    override fun intercept(chain: Interceptor.Chain): Response {
        val originalRequest = chain.request()

        val builder = originalRequest.newBuilder()
            .header("X-Nucleus-Publishable-Key", config.publishableKey)
            .header("User-Agent", "nucleus-android/$SDK_VERSION")
            .header("Accept", "application/json")

        val token = tokenStorage.accessToken
        if (token != null) {
            builder.header("Authorization", "Bearer $token")
        }

        return chain.proceed(builder.build())
    }

    companion object {
        const val SDK_VERSION = "0.1.0"
    }
}
