package dev.nucleus;
public interface NucleusCallback<T> {
    void onSuccess(T result);
    void onError(NucleusException error);
}
