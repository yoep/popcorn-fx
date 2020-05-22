package com.github.yoep.video.javafx.conditions;

import org.springframework.context.annotation.Conditional;

import java.lang.annotation.*;

/**
 * {@link Conditional} that if the JavaFX video player is enabled.
 */
@Target({ ElementType.TYPE, ElementType.METHOD })
@Retention(RetentionPolicy.RUNTIME)
@Documented
@Conditional(OnFXVideoEnabled.class)
public @interface ConditionalOnFXVideoEnabled {
}
