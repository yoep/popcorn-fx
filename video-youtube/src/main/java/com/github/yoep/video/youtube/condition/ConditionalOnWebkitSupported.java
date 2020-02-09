package com.github.yoep.video.youtube.condition;

import org.springframework.context.annotation.Conditional;

import java.lang.annotation.*;

/**
 * {@link Conditional} that verifies if the JFX webkit feature is supported on the platform.
 */
@Target({ ElementType.TYPE, ElementType.METHOD })
@Retention(RetentionPolicy.RUNTIME)
@Documented
@Conditional(OnWebkitSupportedCondition.class)
public @interface ConditionalOnWebkitSupported {
}
