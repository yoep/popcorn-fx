package com.github.yoep.player.qt.conditions;

import org.springframework.context.annotation.Conditional;

import java.lang.annotation.*;

/**
 * {@link Conditional} that only matches when the QT player is enabled.
 */
@Target({ ElementType.TYPE, ElementType.METHOD })
@Retention(RetentionPolicy.RUNTIME)
@Documented
@Conditional(OnQtPlayerEnabledCondition.class)
public @interface ConditionalOnQtPlayerEnabled {
}
