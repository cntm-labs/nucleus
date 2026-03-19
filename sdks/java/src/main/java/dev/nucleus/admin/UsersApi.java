package dev.nucleus.admin;

import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.util.List;
import java.util.Map;

public class UsersApi {
    private final String baseUrl;
    private final String secretKey;
    private final HttpClient httpClient;
    private final ObjectMapper mapper;

    public UsersApi(String baseUrl, String secretKey) {
        this.baseUrl = baseUrl;
        this.secretKey = secretKey;
        this.httpClient = HttpClient.newHttpClient();
        this.mapper = new ObjectMapper();
    }

    public Map<String, Object> getUser(String userId) {
        return request("GET", "/api/v1/users/" + userId, null);
    }

    public List<Map<String, Object>> listUsers(int page, int pageSize) {
        Map<String, Object> result = request("GET",
            "/api/v1/users?page=" + page + "&page_size=" + pageSize, null);
        return mapper.convertValue(result.get("users"), new TypeReference<>() {});
    }

    public Map<String, Object> updateUser(String userId, Map<String, Object> updates) {
        return request("PATCH", "/api/v1/users/" + userId, updates);
    }

    public void deleteUser(String userId) {
        request("DELETE", "/api/v1/users/" + userId, null);
    }

    @SuppressWarnings("unchecked")
    private Map<String, Object> request(String method, String path, Object body) {
        try {
            HttpRequest.Builder reqBuilder = HttpRequest.newBuilder()
                .uri(URI.create(baseUrl + path))
                .header("Authorization", "Bearer " + secretKey)
                .header("Content-Type", "application/json");

            if (body != null) {
                reqBuilder.method(method, HttpRequest.BodyPublishers.ofString(mapper.writeValueAsString(body)));
            } else if ("DELETE".equals(method)) {
                reqBuilder.DELETE();
            } else {
                reqBuilder.GET();
            }

            HttpResponse<String> response = httpClient.send(reqBuilder.build(),
                HttpResponse.BodyHandlers.ofString());

            if (response.statusCode() >= 400) {
                throw new RuntimeException("Nucleus API error: " + response.statusCode() + " " + response.body());
            }

            if (response.body() == null || response.body().isEmpty()) {
                return Map.of();
            }
            return mapper.readValue(response.body(), new TypeReference<>() {});
        } catch (RuntimeException e) {
            throw e;
        } catch (Exception e) {
            throw new RuntimeException("Nucleus API request failed: " + e.getMessage(), e);
        }
    }
}
