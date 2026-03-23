package dev.nucleus.model;

import org.json.JSONObject;

public class NucleusOrganization {
    private final String id;
    private final String name;
    private final String slug;
    private final String createdAt;

    public NucleusOrganization(String id, String name, String slug, String createdAt) {
        this.id = id; this.name = name; this.slug = slug; this.createdAt = createdAt;
    }

    public static NucleusOrganization fromJson(JSONObject json) {
        return new NucleusOrganization(
            json.optString("id"), json.optString("name"),
            json.optString("slug"), json.optString("created_at")
        );
    }

    public String getId() { return id; }
    public String getName() { return name; }
    public String getSlug() { return slug; }
    public String getCreatedAt() { return createdAt; }
}
