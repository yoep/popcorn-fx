package com.github.yoep.video.vlc.conditions;

import org.springframework.context.annotation.Conditional;

import java.lang.annotation.*;

/**
 * {@link Conditional} that only matches when the current CPU architecture is ARM.
 */
@Target({ ElementType.TYPE, ElementType.METHOD })
@Retention(RetentionPolicy.RUNTIME)
@Documented
@Conditional(OnArmCondition.class)
public @interface ConditionalOnArmDevice {
}
