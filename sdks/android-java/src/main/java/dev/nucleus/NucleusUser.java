package dev.nucleus;

public class NucleusUser {
    private final String id;
    private final String email;
    private final String firstName;
    private final String lastName;

    public NucleusUser(String id, String email, String firstName, String lastName) {
        this.id = id; this.email = email; this.firstName = firstName; this.lastName = lastName;
    }

    public String getId() { return id; }
    public String getEmail() { return email; }
    public String getFirstName() { return firstName; }
    public String getLastName() { return lastName; }
    public String getFullName() { return (firstName != null ? firstName : "") + " " + (lastName != null ? lastName : ""); }
}
