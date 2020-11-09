package com.github.yoep.video.vlcnative.conditions;

import org.springframework.context.annotation.Conditional;

import java.lang.annotation.*;

/**
 * {@link Conditional} that only matches when the ARM video player has not been disabled by option {@link Options#DISABLE_ARM_PLAYER}.
 */
@Target({ ElementType.TYPE, ElementType.METHOD })
@Retention(RetentionPolicy.RUNTIME)
@Documented
@Conditional(OnArmVideoEnabled.class)
public @interface ConditionalOnArmVideoEnabled {
}
