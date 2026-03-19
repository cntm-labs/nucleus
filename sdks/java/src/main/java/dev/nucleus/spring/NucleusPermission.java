package dev.nucleus.spring;

import java.lang.annotation.*;

@Target(ElementType.METHOD)
@Retention(RetentionPolicy.RUNTIME)
public @interface NucleusPermission {
    String value();
}
