package com.github.yoep.video.javafx.condition;

import org.springframework.context.annotation.Conditional;

import java.lang.annotation.*;

/**
 * {@link Conditional} that verifies if the JFX media feature is supported on the platform.
 */
@Target({ ElementType.TYPE, ElementType.METHOD })
@Retention(RetentionPolicy.RUNTIME)
@Documented
@Conditional(OnMediaSupportedCondition.class)
public @interface ConditionalOnMediaSupported {
}
