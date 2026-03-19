package dev.nucleus;

import com.auth0.jwt.JWT;
import com.auth0.jwt.algorithms.Algorithm;
import com.auth0.jwt.interfaces.DecodedJWT;
import com.auth0.jwk.JwkProvider;
import com.auth0.jwk.JwkProviderBuilder;
import java.net.URL;
import java.security.interfaces.RSAPublicKey;
import java.util.concurrent.TimeUnit;

public class NucleusTokenVerifier {
    public static NucleusClaims verify(String token, String baseUrl) {
        try {
            JwkProvider provider = new JwkProviderBuilder(new URL(baseUrl + "/.well-known/jwks.json"))
                .cached(10, 24, TimeUnit.HOURS)
                .build();
            DecodedJWT jwt = JWT.decode(token);
            RSAPublicKey publicKey = (RSAPublicKey) provider.get(jwt.getKeyId()).getPublicKey();
            Algorithm algorithm = Algorithm.RSA256(publicKey, null);
            jwt = JWT.require(algorithm).build().verify(token);

            return new NucleusClaims(
                jwt.getSubject(), jwt.getAudience().get(0),
                jwt.getClaim("email").asString(), jwt.getClaim("first_name").asString(),
                jwt.getClaim("last_name").asString(), jwt.getClaim("avatar_url").asString(),
                jwt.getClaim("email_verified").asBoolean(), jwt.getClaim("metadata").asMap(),
                jwt.getClaim("org_id").asString(), jwt.getClaim("org_slug").asString(),
                jwt.getClaim("org_role").asString(), jwt.getClaim("org_permissions").asList(String.class));
        } catch (Exception e) {
            throw new NucleusAuthException("Token verification failed: " + e.getMessage(), e);
        }
    }
}
