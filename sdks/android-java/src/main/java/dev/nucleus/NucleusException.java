package dev.nucleus;

public class NucleusException extends Exception {
    public NucleusException(String message) { super(message); }
    public NucleusException(String message, Throwable cause) { super(message, cause); }
}
