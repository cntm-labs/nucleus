package dev.nucleus;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

class NucleusClientTest {

    @Test
    void builderCreatesClientWithDefaults() {
        NucleusClient client = NucleusClient.builder()
            .secretKey("sk_test_123")
            .build();

        assertNotNull(client);
    }

    @Test
    void builderAcceptsCustomBaseUrl() {
        NucleusClient client = NucleusClient.builder()
            .secretKey("sk_test_123")
            .baseUrl("https://custom.api.dev")
            .build();

        assertNotNull(client);
    }

    @Test
    void usersReturnsNonNull() {
        NucleusClient client = NucleusClient.builder()
            .secretKey("sk_test_123")
            .build();

        assertNotNull(client.users());
    }

    @Test
    void organizationsReturnsNonNull() {
        NucleusClient client = NucleusClient.builder()
            .secretKey("sk_test_123")
            .build();

        assertNotNull(client.organizations());
    }
}
