package dev.nucleus;

import com.auth0.jwt.JWT;
import com.auth0.jwt.algorithms.Algorithm;
import com.github.tomakehurst.wiremock.WireMockServer;
import com.github.tomakehurst.wiremock.core.WireMockConfiguration;
import org.junit.jupiter.api.*;

import java.security.KeyPair;
import java.security.KeyPairGenerator;
import java.security.interfaces.RSAPrivateKey;
import java.security.interfaces.RSAPublicKey;
import java.util.Base64;
import java.util.Date;
import java.util.List;
import java.util.UUID;

import static com.github.tomakehurst.wiremock.client.WireMock.*;
import static org.junit.jupiter.api.Assertions.*;

class NucleusTokenVerifierTest {

    private static WireMockServer wireMock;
    private static RSAPublicKey publicKey;
    private static RSAPrivateKey privateKey;

    @BeforeAll
    static void setup() throws Exception {
        // Generate RSA key pair
        KeyPairGenerator gen = KeyPairGenerator.getInstance("RSA");
        gen.initialize(2048);
        KeyPair pair = gen.generateKeyPair();
        publicKey = (RSAPublicKey) pair.getPublic();
        privateKey = (RSAPrivateKey) pair.getPrivate();

        // Start WireMock
        wireMock = new WireMockServer(WireMockConfiguration.options().dynamicPort());
        wireMock.start();

        // Build JWKS JSON
        String n = Base64.getUrlEncoder().withoutPadding().encodeToString(publicKey.getModulus().toByteArray());
        String e = Base64.getUrlEncoder().withoutPadding().encodeToString(publicKey.getPublicExponent().toByteArray());
        String jwks = String.format(
            """
            {"keys":[{"kty":"RSA","kid":"test-key-1","alg":"RS256","use":"sig","n":"%s","e":"%s"}]}
            """, n, e);

        wireMock.stubFor(get(urlEqualTo("/.well-known/jwks.json"))
            .willReturn(aResponse()
                .withHeader("Content-Type", "application/json")
                .withBody(jwks)));
    }

    @AfterAll
    static void teardown() {
        if (wireMock != null) {
            wireMock.stop();
        }
    }

    private String makeToken(Date expiry) {
        Algorithm alg = Algorithm.RSA256(publicKey, privateKey);
        return JWT.create()
            .withSubject("user_123")
            .withAudience("project_456")
            .withIssuer("https://api.test.com")
            .withIssuedAt(new Date())
            .withExpiresAt(expiry)
            .withJWTId(UUID.randomUUID().toString())
            .withKeyId("test-key-1")
            .withClaim("email", "test@example.com")
            .withClaim("first_name", "Test")
            .withClaim("last_name", "User")
            .sign(alg);
    }

    @Test
    void verifyValidToken() {
        String baseUrl = wireMock.baseUrl();
        Date expiry = new Date(System.currentTimeMillis() + 3600_000);
        String token = makeToken(expiry);

        NucleusClaims claims = NucleusTokenVerifier.verify(token, baseUrl);

        assertEquals("user_123", claims.getUserId());
        assertEquals("project_456", claims.projectId());
        assertEquals("test@example.com", claims.email());
        assertEquals("Test", claims.firstName());
        assertEquals("User", claims.lastName());
    }

    @Test
    void rejectExpiredToken() {
        String baseUrl = wireMock.baseUrl();
        Date expiry = new Date(System.currentTimeMillis() - 3600_000);
        String token = makeToken(expiry);

        assertThrows(NucleusAuthException.class, () ->
            NucleusTokenVerifier.verify(token, baseUrl));
    }

    @Test
    void rejectWrongSignature() throws Exception {
        String baseUrl = wireMock.baseUrl();

        // Generate a different key pair to sign with
        KeyPairGenerator gen = KeyPairGenerator.getInstance("RSA");
        gen.initialize(2048);
        KeyPair wrongPair = gen.generateKeyPair();
        RSAPrivateKey wrongKey = (RSAPrivateKey) wrongPair.getPrivate();

        Algorithm wrongAlg = Algorithm.RSA256(null, wrongKey);
        String token = JWT.create()
            .withSubject("user_123")
            .withAudience("project_456")
            .withExpiresAt(new Date(System.currentTimeMillis() + 3600_000))
            .withKeyId("test-key-1")
            .sign(wrongAlg);

        assertThrows(NucleusAuthException.class, () ->
            NucleusTokenVerifier.verify(token, baseUrl));
    }

    @Test
    void rejectMalformedToken() {
        assertThrows(NucleusAuthException.class, () ->
            NucleusTokenVerifier.verify("not.a.valid.token", wireMock.baseUrl()));
    }
}
