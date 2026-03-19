package dev.nucleus.spring;

import dev.nucleus.NucleusClient;
import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.boot.context.properties.EnableConfigurationProperties;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
@ConditionalOnProperty("nucleus.secret-key")
@EnableConfigurationProperties(NucleusProperties.class)
public class NucleusAutoConfiguration {
    @Bean
    public NucleusClient nucleusClient(NucleusProperties props) {
        return NucleusClient.builder().secretKey(props.getSecretKey()).baseUrl(props.getBaseUrl()).build();
    }
}
