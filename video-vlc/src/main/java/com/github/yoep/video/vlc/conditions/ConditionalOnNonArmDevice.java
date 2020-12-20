package com.github.yoep.video.vlc.conditions;

import org.springframework.context.annotation.Conditional;

import java.lang.annotation.*;

/**
 * {@link Conditional} that matches when the current CPU architecture is non-arm.
 */
@Target({ ElementType.METHOD, ElementType.TYPE })
@Retention(RetentionPolicy.RUNTIME)
@Documented
@Conditional(OnNonArmCondition.class)
public @interface ConditionalOnNonArmDevice {
}
